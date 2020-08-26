#![cfg_attr(not(feature = "SGX_MODE_HW"), allow(unused))]

use std::env;

pub use enclave_ffi_types::ENCRYPTED_SEED_SIZE;
use lazy_static::lazy_static;

#[cfg(all(feature = "production", feature = "SGX_MODE_HW"))]
pub static SPID_FILE: &[u8] = include_bytes!("../spid_production.txt");
#[cfg(all(feature = "production", feature = "SGX_MODE_HW"))]
pub static API_KEY_FILE: &[u8] = include_bytes!("../api_key_production.txt");

#[cfg(all(not(feature = "production"), feature = "SGX_MODE_HW"))]
pub static SPID_FILE: &[u8] = include_bytes!("../spid.txt");
#[cfg(all(not(feature = "production"), feature = "SGX_MODE_HW"))]
pub static API_KEY_FILE: &[u8] = include_bytes!("../api_key.txt");

pub const CERTEXPIRYDAYS: i64 = 90i64;

pub const BECH32_PREFIX_ACC_ADDR: &str = "secret";

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
pub enum SigningMethod {
    MRSIGNER,
    MRENCLAVE,
    NONE,
}

pub const ATTESTATION_CERTIFICATE_SAVE_PATH: &str = "attestation_cert.der";

pub const SEED_EXCH_CERTIFICATE_SAVE_PATH: &str = "node-master-cert.der";
pub const IO_CERTIFICATE_SAVE_PATH: &str = "io-master-cert.der";

//todo: set this to the real value
#[cfg(feature = "production")]
pub const MRSIGNER: [u8; 32] = [
    131, 215, 25, 231, 125, 234, 202, 20, 112, 246, 186, 246, 42, 77, 119, 67, 3, 200, 153, 219,
    105, 2, 15, 156, 112, 238, 29, 252, 8, 199, 206, 158,
];

#[cfg(not(feature = "production"))]
pub const MRSIGNER: [u8; 32] = [
    131, 215, 25, 231, 125, 234, 202, 20, 112, 246, 186, 246, 42, 77, 119, 67, 3, 200, 153, 219,
    105, 2, 15, 156, 112, 238, 29, 252, 8, 199, 206, 158,
];

#[cfg(feature = "production")]
pub const SIGNING_METHOD: SigningMethod = SigningMethod::MRENCLAVE;

#[cfg(not(feature = "production"))]
pub const SIGNING_METHOD: SigningMethod = SigningMethod::MRENCLAVE;

lazy_static! {
    pub static ref CONSENSUS_SEED_SEALING_PATH: String = env::var("SCRT_SGX_STORAGE")
        .unwrap_or_else(|_| "./.sgx_secrets/".to_string())
        + "consensus_seed.sealed";
    pub static ref REGISTRATION_KEY_SEALING_PATH: String = env::var("SCRT_SGX_STORAGE")
        .unwrap_or_else(|_| "./.sgx_secrets/".to_string())
        + "new_node_seed_exchange_keypair.sealed";
}

// pub const REGISTRATION_KEY_SEALING_PATH: &str =
//     "./.sgx_secrets/new_node_seed_exchange_keypair.sealed";

pub const CONSENSUS_SEED_EXCHANGE_KEYPAIR_DERIVE_ORDER: u32 = 1;
pub const CONSENSUS_IO_EXCHANGE_KEYPAIR_DERIVE_ORDER: u32 = 2;
pub const CONSENSUS_STATE_IKM_DERIVE_ORDER: u32 = 3;
