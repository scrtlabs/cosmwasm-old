use std::{
    env,
    path::{Path, PathBuf},
};

use sgx_types::{sgx_attributes_t, sgx_launch_token_t, sgx_misc_attribute_t, SgxResult};
use sgx_urts::SgxEnclave;

pub fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };

    // Step : try to create a .enigma folder for storing all the files
    // Create a directory, returns `io::Result<()>`
    let enclave_directory = env::var("SCRT_ENCLAVE_DIR").unwrap_or('.'.to_string());

    let path = Path::new(&enclave_directory);

    let enclave_file_path: PathBuf = path.join(ENCLAVE_FILE);

    SgxEnclave::create(
        enclave_file_path,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

lazy_static! {
    pub static ref SGX_ENCLAVE: SgxResult<SgxEnclave> = init_enclave();
}
