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

    if args[0] == "-h" {
        println!("Usage: whisk <wasm_file> --invoke <function_name> [--args <arg1> <arg2> ...]");

        return Ok(());
    }

    let wasm_file = &args[0];

    // Parse arguments
    let mut function_name = None;
    let mut function_args = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--invoke" => {
                if i + 1 < args.len() {
                    function_name = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: Missing function name after --invoke");
                    std::process::exit(1);
                }
            }
            "--args" => {
                i += 1;
                while i < args.len() && !args[i].starts_with("--") {
                    if let Ok(value) = args[i].parse::<i32>() {
                        function_args.push(value);
                    } else {
                        eprintln!("Error: Invalid argument '{}'", args[i]);
                        std::process::exit(1);
                    }
                    i += 1;
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    // Read the Wasm file
    let wasm_bytes = read_wasm_file(wasm_file)?;
    if !validate(&wasm_bytes) {
        Err("invalid wasm file")?
    }

    // Skip Wasm header (8 bytes)
    let sections = parser::parse_sections(&wasm_bytes[8..]);

    // Ensure we have a memory section if using WASI
    let memory = sections
        .memory
        .unwrap_or_else(|| types::Memory { min: 1, max: None });

    // Use either --invoke function or fallback to "main"
    let function_to_invoke = function_name.unwrap_or_else(|| "main".to_string());

    // Find the function index in exports
    if let Some(func_index) = sections
        .exports
        .iter()
        .find(|exp| {
            exp.name == function_to_invoke && matches!(exp.kind, types::ExportKind::Function)
        })
        .map(|exp| exp.index)
    {
        let mut ctx = types::Context::new(memory, &sections.funcs);

        if let Some(func) = sections.funcs.get(func_index as usize) {
            // Push function arguments onto the stack
            if !func.params.is_empty() {
                for &arg in &function_args {
                    ctx.stack.push(arg);
                }
            }

            let result = executor::execute_function(&mut ctx, func, &function_args);
            if let Some(val) = result {
                println!("Result: {}", val);
            }
        } else {
            eprintln!(
                "Error: Exported function '{}' not found.",
                function_to_invoke
            );
        }
    } else {
        eprintln!(
            "Error: No exported function '{}' found.",
            function_to_invoke
        );
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
