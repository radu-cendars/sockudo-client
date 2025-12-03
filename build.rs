fn main() {
    // UniFFI scaffolding is now handled via proc macros in the Rust code
    // No build script generation needed for UniFFI 0.30+ with proc macros

    println!("cargo:rerun-if-changed=src/");
}
