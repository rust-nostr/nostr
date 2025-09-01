use std::path::PathBuf;

fn main() {
    rsbinder_aidl::Builder::new()
        .source(PathBuf::from("aidl/INostrSigner.aidl"))
        .output(PathBuf::from("aidl.rs"))
        .generate()
        .unwrap();
}
