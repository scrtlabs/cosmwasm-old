use enclave_ffi_types::EnclaveError;

use crate::crypto::CryptoError;

pub const HMAC_SIGNATURE_SIZE: usize = 32;
pub const EC_256_PRIVATE_KEY_SIZE: usize = 32;

pub trait Encryptable {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

pub trait SIVEncryptable {
    fn encrypt_siv(&self, plaintext: &[u8], ad: Option<&[&[u8]]>) -> Result<Vec<u8>, CryptoError>;
    fn decrypt_siv(&self, plaintext: &[u8], ad: Option<&[&[u8]]>) -> Result<Vec<u8>, CryptoError>;
}

pub trait SealedKey
where
    Self: std::marker::Sized,
{
    fn seal(&self, filepath: &str) -> Result<(), EnclaveError>;
    fn unseal(filepath: &str) -> Result<Self, EnclaveError>;
}

pub trait Rng {
    fn rand_slice(buf: &mut [u8]) -> Result<(), CryptoError>;
}

pub trait Kdf<T> {
    fn derive_key_from_this(&self, data: &[u8]) -> T;
}

pub trait Hmac {
    fn sign_sha_256(&self, to_sign: &[u8]) -> [u8; HMAC_SIGNATURE_SIZE];
}

pub trait AlignedMemory {}

pub trait ExportECKey {
    fn key_ref(&self) -> &[u8; EC_256_PRIVATE_KEY_SIZE];
}
