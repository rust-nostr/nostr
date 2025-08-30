use std::path::PathBuf;

fn main() {
    rsbinder_aidl::Builder::new()
        .source(PathBuf::from("aidl/ISigner.aidl"))
        .output(PathBuf::from("aidl_signer.rs"))
        .generate()
        .unwrap();
}
