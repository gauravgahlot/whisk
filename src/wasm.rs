use std::collections::HashMap;

/// 4-byte magic number. The string `\0asm`.
const MAGIC_NUMBER: &[u8] = &[0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version. Current version is 1.
const WASM_BIN_FMT_VERSION: &[u8] = &[0x1, 0x00, 0x00, 0x00];

pub(crate) fn validate(bytes: &[u8]) -> bool {
    &bytes[0..4] == MAGIC_NUMBER && &bytes[4..8] == WASM_BIN_FMT_VERSION
}

pub(crate) fn parse(bytes: &[u8]) -> HashMap<u8, Vec<u8>> {
    let mut sections: HashMap<u8, Vec<u8>> = HashMap::new();
    let mut idx = 0;

    while idx < bytes.len() {
        let section_id = bytes[idx];
        idx += 1;

        let (payload_len, len_bytes) = decode_leb128(&bytes[idx..]);
        idx += len_bytes;

        let payload = bytes[idx..idx + payload_len as usize].to_vec();
        idx += payload_len as usize;

        sections.insert(section_id, payload);
    }

    sections
}

pub(crate) fn parse_type_section(payload: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut functions = Vec::new();
    let mut idx = 0;

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

fn decode_leb128(bytes: &[u8]) -> (u32, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut count = 0;

    for &byte in bytes {
        result |= ((byte & 0x7F) as u32) << shift; // Take 7 bits and shift them into position
        shift += 7;
        count += 1;

        if byte & 0x80 == 0 {
            // If MSB is 0, we've reached the last byte
            break;
        }
    }

    (result, count) // Return the decoded value and the number of bytes used
}
