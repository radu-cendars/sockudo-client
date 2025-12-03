use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Filter out --target flag which causes issues with emcc
    let filtered_args: Vec<String> = args
        .iter()
        .skip(1) // Skip program name
        .filter(|arg| !arg.starts_with("--target"))
        .map(|s| s.to_string())
        .collect();

    // Call emcc with filtered arguments
    let emcc_path = "C:/emsdk/upstream/emscripten/emcc.bat";

    let status = Command::new(emcc_path)
        .args(&filtered_args)
        .status()
        .expect("Failed to execute emcc");

    std::process::exit(status.code().unwrap_or(1));
}
