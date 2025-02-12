use std::vec;

use crate::{leb128, types};

pub(crate) fn parse_sections(bytes: &[u8]) -> types::Sections {
    let mut sections = types::Sections::default();
    let mut idx = 0;

    while idx < bytes.len() {
        let section_id = bytes[idx];
        idx += 1;

        let (section_size, len_bytes) = leb128::decode(&bytes[idx..]);
        idx += len_bytes;

        let payload = &bytes[idx..idx + section_size as usize];
        idx += section_size as usize;

        match section_id {
            1 => sections.types = parse_type_section(payload),
            2 => sections.imports = parse_import_section(payload),
            3 => sections.functions = parse_function_section(payload),
            7 => sections.exports = parse_export_section(payload),
            10 => {
                sections.funcs = parse_code_section(payload, &sections.functions, &sections.types)
            }
            _ => {} // Ignore other sections for now
        }
    }

    sections
}

pub(crate) fn parse_type_section(payload: &[u8]) -> Vec<types::FuncSignature> {
    let mut funcs = Vec::new();
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

        let returns_count = payload[idx] as usize;
        idx += 1;
        let returns = payload[idx..idx + returns_count].to_vec();
        idx += returns_count;

        funcs.push(types::FuncSignature { params, returns });
    }

    funcs
}

pub(crate) fn parse_import_section(payload: &[u8]) -> Vec<types::Import> {
    let mut imports = Vec::new();
    let mut idx = 0;

    let (import_count, len_bytes) = leb128::decode(&payload[idx..]);
    idx += len_bytes;

    for _ in 0..import_count {
        // read module name
        let (module, bytes_read) = read_wasm_string(&payload[idx..]);
        idx += bytes_read;

        // read import name
        let (name, bytes_read) = read_wasm_string(&payload[idx..]);
        idx += bytes_read;

        // read the import kind
        let kind = match payload[idx] {
            0x00 => types::ImportKind::Function,
            0x01 => types::ImportKind::Table,
            0x02 => types::ImportKind::Memory,
            0x03 => types::ImportKind::Global,
            _ => panic!("Unknown import kind: {}", payload[idx]),
        };
        idx += 1;

        imports.push(types::Import { module, name, kind });
    }

    imports
}

pub(crate) fn parse_function_section(payload: &[u8]) -> Vec<u32> {
    let mut idx = 0;
    let (function_count, len_bytes) = leb128::decode(&payload[idx..]);
    idx += len_bytes;

    let mut type_indices = Vec::new();
    for _ in 0..function_count {
        let (type_index, type_bytes) = leb128::decode(&payload[idx..]);
        idx += type_bytes;
        type_indices.push(type_index);
    }

    type_indices
}

pub(crate) fn parse_export_section(payload: &[u8]) -> Vec<types::Export> {
    let mut exports = Vec::new();
    let mut idx = 0;

    let count = payload[idx] as usize;
    idx += 1;

    for _ in 0..count {
        let (name_len, len_bytes) = leb128::decode(&payload[idx..]);
        idx += len_bytes;

        let name = String::from_utf8(payload[idx..idx + name_len as usize].to_vec())
            .expect("Invalid UTF-8 in export name");
        idx += name_len as usize;

        let kind = match payload[idx] {
            0x00 => types::ExportKind::Function,
            0x01 => types::ExportKind::Table,
            0x02 => types::ExportKind::Memory,
            0x03 => types::ExportKind::Global,
            _ => panic!("Unknown export kind"),
        };
        idx += 1;

        let (index, len_bytes) = leb128::decode(&payload[idx..]);
        idx += len_bytes;

        exports.push(types::Export {
            name,
            kind,
            index: index as usize,
        });
    }

    exports
}

pub(crate) fn parse_code_section(
    payload: &[u8],
    function_indices: &[u32],
    type_section: &[types::FuncSignature],
) -> Vec<types::Func> {
    let mut funcs = Vec::new();
    let mut idx = 0;

    // number of function bodies
    let (entry_count, len_bytes) = leb128::decode(&payload[idx..]);
    idx += len_bytes;

    for i in 0..entry_count {
        // function body length
        let (entry_len, len_bytes) = leb128::decode(&payload[idx..]);
        idx += len_bytes;

        let (locals, locals_bytes) = parse_locals(&payload[idx..]);
        idx += locals_bytes;

        let body = payload[idx..idx + (entry_len as usize - locals_bytes)].to_vec();
        idx += entry_len as usize - locals_bytes;

        // Fetch signature index (default to 0 if missing)
        let signature_index = function_indices.get(i as usize).copied().unwrap_or(0);

        // Fetch params from type section
        let params = type_section
            .get(signature_index as usize)
            .map(|sig| {
                sig.params
                    .iter()
                    .map(|&b| types::ValType::from(b))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        funcs.push(types::Func {
            params,
            locals,
            body,
            signature_index,
        });
    }

    funcs
}

fn read_wasm_string(bytes: &[u8]) -> (String, usize) {
    let (length, len_bytes) = leb128::decode(bytes);
    let string_bytes = &bytes[len_bytes..len_bytes + length as usize];
    let string = String::from_utf8(string_bytes.to_vec()).expect("Invalid UTF-8 string");
    (string, len_bytes + length as usize)
}

fn parse_locals(payload: &[u8]) -> (Vec<(u32, u8)>, usize) {
    let mut locals = Vec::new();
    let mut idx = 0;

    let (local_count, len_bytes) = leb128::decode(&payload[idx..]); // Number of local declarations
    idx += len_bytes;

    for _ in 0..local_count {
        let (count, count_bytes) = leb128::decode(&payload[idx..]); // Number of variables
        idx += count_bytes;

        let value_type = payload[idx]; // Variable type (e.g., 0x7F for i32)
        idx += 1;

        locals.push((count, value_type));
    }

    (locals, idx) // Return parsed locals + bytes consumed
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

        // Parse the sections
        let sections = parse_sections(&wasm_binary[8..]); // Skip the header

        // Verify Type Section
        assert_eq!(sections.types.len(), 1);
        assert_eq!(sections.types[0].params, vec![]);
        assert_eq!(sections.types[0].returns, vec![0x7F]); // i32 return type

        // Verify Function Section
        assert_eq!(sections.functions.len(), 1);
        assert_eq!(sections.functions[0], 0); // Function index 0 has type index 0

        // Verify Code Section
        assert_eq!(sections.funcs.len(), 2);

        // First function: local.get 0, local.get 1, i32.add, end
        assert_eq!(sections.funcs[0].locals.len(), 0);
        assert_eq!(
            sections.funcs[0].body,
            vec![0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B]
        );

        // Second function: local.get 0, i32.const 1, call 0, end
        assert_eq!(sections.funcs[1].locals.len(), 0);
        assert_eq!(
            sections.funcs[1].body,
            vec![0x20, 0x00, 0x41, 0x01, 0x10, 0x00, 0x0B]
        );
    }
}
