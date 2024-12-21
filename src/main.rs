mod leb128;
mod parser;

use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = read_wasm_file("bin/wat/plus_one.wasm")?;
    if !parser::validate(&wasm_bytes) {
        Err("invalid wasm file")?
    }

    // skip wasm header (8 bytes)
    let sections = parser::parse_sections(&wasm_bytes[8..]);
    // for (id, sec) in &sections {
    //     println!("section {:?}\n{:?}\n", id, sec);
    // }

    // parse type section
    if let Some(type_payload) = sections.get(&1) {
        let type_info = parser::parse_type_section(type_payload);
        println!("type section: \n\t{:?}", type_info);
    }

    // parse function section
    if let Some(func_payload) = sections.get(&3) {
        let funcs = parser::parse_function_section(func_payload);
        println!("function section: \n\t{:?}", funcs);
    }

    // parse code section
    if let Some(code_payload) = sections.get(&10) {
        let entries = parser::parse_code_section(code_payload);
        println!("code sections:");
        for e in &entries {
            println!("\t{:?}", e);
        }
    }

    Ok(())
}

fn read_wasm_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
