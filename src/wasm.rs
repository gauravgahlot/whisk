/// 4-byte magic number. The string `\0asm`.
const MAGIC_NUMBER: &[u8] = &[0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version. Current version is 1.
const WASM_BIN_FMT_VERSION: &[u8] = &[0x1, 0x00, 0x00, 0x00];

pub(crate) fn validate(bytes: &[u8]) -> bool {
    &bytes[0..4] == MAGIC_NUMBER && &bytes[4..8] == WASM_BIN_FMT_VERSION
}
