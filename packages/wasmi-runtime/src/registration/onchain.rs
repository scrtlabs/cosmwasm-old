///
/// These functions run on-chain and must be deterministic across all nodes
///
use log::*;
use std::panic;

use enclave_ffi_types::NodeAuthResult;

use crate::consts::ENCRYPTED_SEED_SIZE;
use crate::crypto::PUBLIC_KEY_SIZE;
use crate::{
    oom_handler::{get_then_clear_oom_happened, register_oom_handler},
    utils::{validate_const_ptr, validate_mut_ptr},
};

use super::cert::verify_ra_cert;
use super::seed_exchange::encrypt_seed;

///
/// `ecall_authenticate_new_node`
///
/// This call is used to help new nodes register in the network. The function will authenticate the
/// new node, based on a received certificate. If the node is authenticated successfully, the seed
/// will be encrypted and shared with the registering node.
///
/// The seed is encrypted with a key derived from the secret master key of the chain, and the public
/// key of the requesting chain
///
/// This function happens on-chain, so any panic here might cause the chain to go boom
///
/// # Safety
/// Safety first
#[no_mangle]
pub unsafe extern "C" fn ecall_authenticate_new_node(
    cert: *const u8,
    cert_len: u32,
    seed: &mut [u8; ENCRYPTED_SEED_SIZE],
) -> NodeAuthResult {
    register_oom_handler();

    if let Err(_e) = validate_mut_ptr(seed.as_mut_ptr(), seed.len()) {
        return NodeAuthResult::InvalidInput;
    }
    if let Err(_e) = validate_const_ptr(cert, cert_len as usize) {
        return NodeAuthResult::InvalidInput;
    }
    let cert_slice = std::slice::from_raw_parts(cert, cert_len as usize);

    let result = panic::catch_unwind(|| -> Result<Vec<u8>, NodeAuthResult> {
        // verify certificate, and return the public key in the extra data of the report
        let pk = verify_ra_cert(cert_slice)?;

        // just make sure the length isn't wrong for some reason (certificate may be malformed)
        if pk.len() != PUBLIC_KEY_SIZE {
            error!(
                "Got public key from certificate with the wrong size: {:?}",
                pk.len()
            );
            return Err(NodeAuthResult::MalformedPublicKey);
        }

        let mut target_public_key: [u8; 32] = [0u8; 32];
        target_public_key.copy_from_slice(&pk);
        debug!(
            "ecall_get_encrypted_seed target_public_key key pk: {:?}",
            &target_public_key.to_vec()
        );

        let res: Vec<u8> =
            encrypt_seed(target_public_key).map_err(|_| NodeAuthResult::SeedEncryptionFailed)?;

        Ok(res)
    });

    if let Ok(res) = result {
        match res {
            Ok(res) => {
                seed.copy_from_slice(&res);
                NodeAuthResult::Success
            }
            Err(e) => e,
        }
    } else {
        // There's no real need here to test if oom happened
        get_then_clear_oom_happened();
        error!("Enclave call ecall_authenticate_new_node panic!");
        NodeAuthResult::Panic
    }
}
