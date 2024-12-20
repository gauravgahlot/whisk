/// 4-byte magic number. The string `\0asm`.
const MAGIC_NUMBER: &[u8] = &[0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version. Current version is 1.
const WASM_BIN_FMT_VERSION: &[u8] = &[0x1, 0x00, 0x00, 0x00];

pub(crate) fn validate(bytes: &[u8]) -> bool {
    &bytes[0..4] == MAGIC_NUMBER && &bytes[4..8] == WASM_BIN_FMT_VERSION
}

pub(crate) fn parse(mut bytes: &[u8]) {
    // skip the 8-byte Wasm header
    bytes = &bytes[8..];

    while !bytes.is_empty() {
        let section_id = &bytes[0];
        let (content_len, len_bytes) = decode_leb128(&bytes[1..]);

        // skip section_id and bytes required to encode content_len
        let content_start = 1 + len_bytes;
        let content_end = content_start + content_len;

        // read the content
        let _content = &bytes[content_start..content_end];

        match section_id {
            1 => println!("type section found"),
            3 => println!("function section found"),
            10 => println!("code section found"),
            _ => println!("unknown section: {}", section_id),
        }

        // move to the next section
        bytes = &bytes[content_end..];
    }
}

fn decode_leb128(mut bytes: &[u8]) -> (usize, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut size = 0;

    while let Some(&byte) = bytes.first() {
        bytes = &bytes[1..];
        size += 1;

        result |= ((byte & 0x7F) as usize) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    (result, size) // Return decoded value and size in bytes
}
