mod executor;
mod leb128;
mod parser;
mod registry;
mod wasi;

use std::env;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() == 0 {
        return Ok(());
    }

    if args.len() > 1 {
        println!("exepected exactly one argument");
        return Ok(());
    }

    let wasm_bytes = read_wasm_file(args[0].as_str())?;
    if !parser::validate(&wasm_bytes) {
        Err("invalid wasm file")?
    }

    // skip wasm header (8 bytes)
    let sections = parser::parse_sections(&wasm_bytes[8..]);

    // Extract imports, functions, and memory
    let mut registry = registry::ImportRegistry::new();
    registry.add_function("wasi_snapshot_preview1", "fd_write", wasi::fd_write);

    let mut memory_size = 0;
    if let Some(memory_section) = sections.get(&5) {
        memory_size = parser::parse_memory_section(memory_section);
    }

    // parse type section
    if let Some(type_payload) = sections.get(&1) {
        let _type_info = parser::parse_type_section(type_payload);
    }

    // parse function section
    if let Some(func_payload) = sections.get(&3) {
        let _funcs = parser::parse_function_section(func_payload);
    }

    // parse import section
    if let Some(import_payload) = sections.get(&2) {
        println!("{:?}", import_payload);
        let _imports = parser::parse_import_section(import_payload);
    }

    // parse code section
    if let Some(code_payload) = sections.get(&10) {
        let entries = parser::parse_code_section(code_payload);
        for e in &entries {
            let func = executor::Func::new(e.0.clone(), e.1.clone());
            let mut ctx = executor::Context::new(memory_size);

            let result = executor::execute_function(&mut ctx, &func, &registry);
            match result {
                Some(v) => println!("{}", v),
                None => continue,
            };
        }
    }

    let mut ctx = executor::Context::new(memory_size);
    let start_func_idx = parser::parse_exports_section(sections.get(&7).unwrap())
        .iter()
        .find(|export| export.name == "_start" && export.kind == parser::ExportKind::Function)
        .expect("No _start function found")
        .index;

    Ok(())
}

fn read_wasm_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
