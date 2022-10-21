// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

fn main() {
    uniffi_build::generate_scaffolding("./src/api.udl").expect("Building the UDL file failed");
}
