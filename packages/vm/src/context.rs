/**
Internal details to be used by instance.rs only
**/
#[cfg(feature = "iterator")]
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::c_void;
use std::mem;

use wasmer_runtime_core::vm::Ctx;

use cosmwasm_std::{Api, Binary, CanonicalAddr, HumanAddr, Storage};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, Pair};

use crate::errors::Error;
#[cfg(feature = "iterator")]
use crate::memory::maybe_read_region;
use crate::memory::{read_region, write_region};

/// An unknown error occurred when writing to region
static ERROR_WRITE_TO_REGION_UNKNONW: i32 = -1000001;
/// Could not write to region because it is too small
static ERROR_WRITE_TO_REGION_TOO_SMALL: i32 = -1000002;

/// Invalid Order enum value passed into scan
static ERROR_SCAN_INVALID_ORDER: i32 = -2000001;
// Iterator pointer not registered
static ERROR_NEXT_INVALID_ITERATOR: i32 = -2000002;
/// Generic error - using context with no Storage attached
static ERROR_NO_STORAGE: i32 = -3000001;

/// Reads a storage entry from the VM's storage into Wasm memory
pub fn do_read<T: Storage>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = read_region(ctx, key_ptr);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context(ctx, |store: &mut T| value = store.get(&key));
    match value {
        Some(buf) => match write_region(ctx, value_ptr, &buf) {
            Ok(()) => 0,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
            Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
        },
        None => 0,
    }
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_write<T: Storage>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) {
    let key = read_region(ctx, key_ptr);
    let value = read_region(ctx, value_ptr);
    with_storage_from_context(ctx, |store: &mut T| store.set(&key, &value));
}

pub fn do_remove<T: Storage>(ctx: &Ctx, key_ptr: u32) {
    let key = read_region(ctx, key_ptr);
    with_storage_from_context(ctx, |store: &mut T| store.remove(&key));
}

#[cfg(feature = "iterator")]
pub fn do_scan<T: Storage>(ctx: &Ctx, start_ptr: u32, end_ptr: u32, order: i32) -> i32 {
    let start = maybe_read_region(ctx, start_ptr);
    let end = maybe_read_region(ctx, end_ptr);
    let order: Order = match order.try_into() {
        Ok(o) => o,
        Err(_) => return ERROR_SCAN_INVALID_ORDER,
    };
    let mut storage: Option<T> = take_storage(ctx);
    if let Some(store) = &mut storage {
        let iter = store.range(start.as_deref(), end.as_deref(), order);
        let res = leave_iterator::<T>(ctx, iter, None);
        leave_storage(ctx, storage);
        return res;
    } else {
        leave_storage(ctx, storage);
        return ERROR_NO_STORAGE;
    }
}

#[cfg(feature = "iterator")]
pub fn do_next<T: Storage>(ctx: &Ctx, iter_ptr: i32, key_ptr: u32, value_ptr: u32) -> i32 {
    let mut iter = match take_iterator::<T>(ctx, iter_ptr) {
        Some(i) => i,
        None => return ERROR_NEXT_INVALID_ITERATOR,
    };
    // get next item and return iterator
    let item = iter.next();
    leave_iterator::<T>(ctx, iter, Some(iter_ptr));

    // prepare return values
    let (key, value) = match item {
        Some(item) => item,
        None => {
            return 0;
        }
    };
    match write_region(ctx, key_ptr, &key) {
        // TODO: we need to return the length of key somehow, now this is just the length of value
        Ok(_) => {}
        Err(Error::RegionTooSmallErr { .. }) => return ERROR_WRITE_TO_REGION_TOO_SMALL,
        Err(_) => return ERROR_WRITE_TO_REGION_UNKNONW,
    };
    match write_region(ctx, value_ptr, &value) {
        Ok(bytes_written) => bytes_written.try_into().unwrap(),
        Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
        Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
    }
}

pub fn do_canonical_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    human_ptr: u32,
    canonical_ptr: u32,
) -> i32 {
    let human = read_region(ctx, human_ptr);
    let human = match String::from_utf8(human) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return -2,
    };
    match api.canonical_address(&human) {
        Ok(canon) => match write_region(ctx, canonical_ptr, canon.as_slice()) {
            Ok(()) => 0,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
            Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
        },
        Err(_) => -1,
    }
}

pub fn do_human_address<A: Api>(api: A, ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32) -> i32 {
    let canon = Binary(read_region(ctx, canonical_ptr));
    match api.human_address(&CanonicalAddr(canon)) {
        Ok(human) => match write_region(ctx, human_ptr, human.as_str().as_bytes()) {
            Ok(()) => 0,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
            Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
        },
        Err(_) => -1,
    }
}

/** context data **/

struct ContextData<S: Storage> {
    data: Option<S>,
    #[cfg(feature = "iterator")]
    iters: IterInfo,
}

#[cfg(feature = "iterator")]
struct IterInfo {
    counter: i32,
    lookup: HashMap<i32, Box<dyn Iterator<Item = Pair>>>,
}

#[cfg(feature = "iterator")]
impl IterInfo {
    fn new() -> Self {
        IterInfo {
            counter: 0,
            lookup: HashMap::new(),
        }
    }
}

pub fn setup_context<S: Storage>() -> (*mut c_void, fn(*mut c_void)) {
    (
        create_unmanaged_storage::<S>(),
        destroy_unmanaged_storage::<S>,
    )
}

fn create_unmanaged_storage<S: Storage>() -> *mut c_void {
    let data = ContextData::<S> {
        data: None,
        #[cfg(feature = "iterator")]
        iters: IterInfo::new(),
    };
    let state = Box::new(data);
    Box::into_raw(state) as *mut c_void
}

unsafe fn get_data<S: Storage>(ptr: *mut c_void) -> Box<ContextData<S>> {
    Box::from_raw(ptr as *mut ContextData<S>)
}

fn destroy_unmanaged_storage<S: Storage>(ptr: *mut c_void) {
    if !ptr.is_null() {
        // auto-dropped with scope
        // TODO: do we need to manually clean up iterators before storage?
        let _ = unsafe { get_data::<S>(ptr) };
    }
}

pub fn with_storage_from_context<S: Storage, F: FnMut(&mut S)>(ctx: &Ctx, mut func: F) {
    let mut storage: Option<S> = take_storage(ctx);
    if let Some(data) = &mut storage {
        func(data);
    }
    leave_storage(ctx, storage);
}

pub fn take_storage<S: Storage>(ctx: &Ctx) -> Option<S> {
    let mut b = unsafe { get_data(ctx.data) };
    let res = b.data.take();
    mem::forget(b); // we do this to avoid cleanup
    res
}

pub fn leave_storage<S: Storage>(ctx: &Ctx, storage: Option<S>) {
    let mut b = unsafe { get_data(ctx.data) };
    // clean-up if needed
    let _ = b.data.take();
    b.data = storage;
    mem::forget(b); // we do this to avoid cleanup
}

#[cfg(feature = "iterator")]
// if ptr is None, find a new slot.
// otherwise, place in slot defined by ptr (only after take)
pub fn leave_iterator<S: Storage>(
    ctx: &Ctx,
    iter: Box<dyn Iterator<Item = Pair>>,
    ptr: Option<i32>,
) -> i32 {
    let mut b = unsafe { get_data::<S>(ctx.data) };
    let ptr = match ptr {
        Some(v) => v,
        None => {
            b.iters.counter += 1;
            b.iters.counter
        }
    };
    b.iters.lookup.insert(ptr, iter);
    ptr
}

#[cfg(feature = "iterator")]
pub fn take_iterator<S: Storage>(ctx: &Ctx, ptr: i32) -> Option<Box<dyn Iterator<Item = Pair>>> {
    let mut b = unsafe { get_data::<S>(ctx.data) };
    b.iters.lookup.remove(&ptr)
}
