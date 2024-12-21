use super::leb128;

use std::collections::HashMap;

/// 4-byte magic number. The string `\0asm`.
const MAGIC_NUMBER: &[u8] = &[0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version. Current version is 1.
const WASM_BIN_FMT_VERSION: &[u8] = &[0x1, 0x00, 0x00, 0x00];

pub(crate) fn validate(bytes: &[u8]) -> bool {
    &bytes[0..4] == MAGIC_NUMBER && &bytes[4..8] == WASM_BIN_FMT_VERSION
}

pub(crate) fn parse_sections(bytes: &[u8]) -> HashMap<u8, Vec<u8>> {
    let mut sections: HashMap<u8, Vec<u8>> = HashMap::new();
    let mut idx = 0;

    while idx < bytes.len() {
        if idx >= bytes.len() {
            break; // Prevent out-of-bounds access
        }

        let section_id = bytes[idx];
        idx += 1;

        // Decode the LEB128 payload length
        if idx >= bytes.len() {
            break; // Prevent out-of-bounds access
        }

        let (payload_len, len_bytes) = leb128::decode(&bytes[idx..]);
        idx += len_bytes;

        // Ensure we have enough bytes for the payload
        if idx + payload_len as usize > bytes.len() {
            panic!(
                "Invalid payload length: idx={}, payload_len={}, bytes.len()={}",
                idx,
                payload_len,
                bytes.len()
            );
        }

        // Extract the payload
        let payload = bytes[idx..idx + payload_len as usize].to_vec();
        idx += payload_len as usize;

        // Insert the section into the map
        sections.insert(section_id, payload);
    }

    sections
}

pub(crate) fn parse_type_section(payload: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut functions = Vec::new();
    let mut idx = 0;

    if payload.len() == 0 {
        return vec![];
    }

    let num_types = payload[idx];
    idx += 1; // move past the type count

    for _ in 0..num_types {
        let func_type = payload[idx];
        idx += 1;

        // continue if not a function type
        if func_type != 0x60 {
            continue;
        }

        let params_count = payload[idx] as usize;
        idx += 1;
        let params = payload[idx..idx + params_count].to_vec();
        idx += params_count;

        let result_count = payload[idx] as usize;
        idx += 1;
        let results = payload[idx..idx + result_count].to_vec();
        idx += result_count;

        functions.push((params, results));
    }

    functions
}

pub(crate) fn parse_function_section(payload: &[u8]) -> Vec<u32> {
    let mut functions = Vec::new();
    let mut idx = 0;

    let fn_count = payload[idx] as usize;
    idx += 1; // move past the function count

    for _ in 0..fn_count {
        let type_index = payload[idx] as u32;
        idx += 1;
        functions.push(type_index);
    }

    functions
}

/// Returns Vec<(Vec<(locals_count, locals_type)>, Vec<instructions>)>
pub(crate) fn parse_code_section(payload: &[u8]) -> Vec<(Vec<(u32, u8)>, Vec<u8>)> {
    let mut entries = Vec::new();

    let mut idx = 0;
    let entry_count = payload[idx];
    idx += 1;

    for _ in 0..entry_count {
        // decode the size of the function entry
        let (fn_len, len_bytes) = leb128::decode(&payload[idx..]);
        idx += len_bytes;

        // extract the function body
        let fn_body = &payload[idx..idx + fn_len as usize];
        idx += fn_len as usize;

        // parse function body
        let mut bidx = 0;

        // parse the local count
        let (locals_count, len_bytes) = leb128::decode(&fn_body[bidx..]);
        bidx += len_bytes;

        let mut locals = vec![];
        for _ in 0..locals_count {
            // parse each local declaration (count, type)
            let (count, count_bytes) = leb128::decode(&fn_body[bidx..]);
            bidx += count_bytes;

            // 1 byte for the type (e.g., 0x7F for i32, 0x7E for i64)
            let local_type = fn_body[bidx];
            bidx += 1;

            locals.push((count, local_type));
        }

        // rest is the instruction sequence
        let instructions = fn_body[bidx..].to_vec();

        // store the locals, and instructions for the entry
        entries.push((locals, instructions))
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sections() {
        // Mock WASM binary with header, Type, Function, and Code sections
        let wasm_binary = [
            // 8-byte WASM header
            0x00, 0x61, 0x73, 0x6D, // Magic number '\0asm'
            0x01, 0x00, 0x00, 0x00, // Version: 1
            // Type Section
            0x01, // Section ID: Type
            0x05, // Section size: 5 bytes
            0x01, // One type entry
            0x60, 0x00, 0x01, 0x7F, // Function type: () -> i32
            // Function Section
            0x03, // Section ID: Function
            0x02, // Section size: 2 bytes
            0x01, // One function
            0x00, // Function index 0 refers to type 0
            // Code Section
            0x0A, // Section ID: Code
            0x12, // Section size: 12 bytes
            0x02, // Two function bodies
            // First function body
            0x07, // Size of the first function body: 9 bytes
            0x00, // Local count: 0
            0x20, 0x00, 0x20, 0x01, 0x6A,
            0x0B, // Instructions: local.get 0, local.get 1, i32.add, end
            // Second function body
            0x08, // Size of the second function body: 8 bytes
            0x00, // Local count: 0
            0x20, 0x00, 0x41, 0x01, 0x10, 0x00,
            0x0B, // Instructions: local.get 0, i32.const 1, call 0, end
        ];

        // Validate the binary
        assert!(validate(&wasm_binary));

        // Parse the sections
        let sections = parse_sections(&wasm_binary[8..]); // Skip the header

        // Verify Type Section
        let type_section = sections.get(&0x01).unwrap();
        let types = parse_type_section(type_section);
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].0.len(), 0); // No parameters
        assert_eq!(types[0].1.len(), 1); // One result
        assert_eq!(types[0].1[0], 0x7F); // i32 result

        // Verify Function Section
        let function_section = sections.get(&0x03).unwrap();
        let functions = parse_function_section(function_section);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0], 0);

        // Verify Code Section
        let code_section = sections.get(&0x0A).unwrap();
        let code_entries = parse_code_section(code_section);
        assert_eq!(code_entries.len(), 2);

        // First function body
        let (locals, instructions) = &code_entries[0];
        assert_eq!(locals.len(), 0);
        assert_eq!(instructions, &[0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B]);

        // Second function body
        let (locals, instructions) = &code_entries[1];
        assert_eq!(locals.len(), 0); // No locals
        assert_eq!(instructions, &[0x20, 0x00, 0x41, 0x01, 0x10, 0x00, 0x0B]);
    }
}
