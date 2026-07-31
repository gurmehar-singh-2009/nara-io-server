use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // compile time seed based on time
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let seed = (nanos & 0xFF) as u8;

    println!("cargo:rustc-env=PACKET_SEED={}", seed);
    println!("cargo:rerun-if-changed=build.rs");
}
