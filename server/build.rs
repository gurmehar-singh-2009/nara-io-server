use ed25519_dalek::SigningKey;
use getrandom::{SysRng, rand_core::UnwrapErr};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap();

    let secret_key_path = workspace_root.join("server").join("server_key.bin");
    let public_key_path = workspace_root.join("client").join("public_key.bin");

    if !secret_key_path.exists() || !public_key_path.exists() {
        let mut rng = UnwrapErr(SysRng);
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        fs::write(&secret_key_path, signing_key.to_bytes())
            .expect("failed to write secret key to server/");

        fs::write(&public_key_path, verifying_key.to_bytes())
            .expect("failed to write public key to client/");
    }

    println!("cargo:rerun-if-changed={}", secret_key_path.display());
}
