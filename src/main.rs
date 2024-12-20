mod wasm;

use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = read_wasm_file("bin/hello.wasm")?;
    if !wasm::validate(&wasm_bytes) {
        Err("invalid wasm file")?
    }

    println!("valid wasm file");

    Ok(())
}

fn read_wasm_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
