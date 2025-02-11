mod executor;
mod leb128;
mod parser;
mod types;

use std::env;
use std::io::Read;

/// 4-byte magic number. The string `\0asm`.
const MAGIC_NUMBER: &[u8] = &[0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version. Current version is 1.
const WASM_BIN_FMT_VERSION: &[u8] = &[0x1, 0x00, 0x00, 0x00];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() < 2 {
        eprintln!("Usage: whisk <wasm_file>");
        std::process::exit(1)
    }

    // read the wasm file
    let wasm_bytes = read_wasm_file(args[0].as_str())?;
    if !validate(&wasm_bytes) {
        Err("invalid wasm file")?
    }

    // skip wasm header (8 bytes)
    let sections = parser::parse_sections(&wasm_bytes[8..]);

    // ensure we have a memory section if using WASI
    let memory = sections
        .memory
        .unwrap_or_else(|| types::Memory { min: 1, max: None });

    // Find the "main" function in the exports
    if let Some(main_index) = sections
        .exports
        .iter()
        .find(|exp| exp.name == "main" && matches!(exp.kind, types::ExportKind::Function))
        .map(|exp| exp.index)
    {
        let mut ctx = types::Context::new(memory);

        if let Some(func) = sections.funcs.get(main_index as usize) {
            let result = executor::execute_function(&mut ctx, func);
            if let Some(val) = result {
                println!("{}", val);
            }
        } else {
            eprintln!("Error: Exported function 'main' not found.");
        }
    } else {
        eprintln!("Error: No exported function 'main' found.");
    }

    Ok(())
}

fn read_wasm_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn validate(bytes: &[u8]) -> bool {
    &bytes[0..4] == MAGIC_NUMBER && &bytes[4..8] == WASM_BIN_FMT_VERSION
}
