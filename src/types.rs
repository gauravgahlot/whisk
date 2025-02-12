#[derive(Default)]
pub(crate) struct Sections {
    pub types: Vec<FuncSignature>, // Parsed function signatures
    pub functions: Vec<u32>,       // Type indices for functions
    pub exports: Vec<Export>,      // Parsed exports
    pub imports: Vec<Import>,      // Parsed imports
    pub funcs: Vec<Func>,          // Parsed function bodies
    pub memory: Option<Memory>,    // Memory section if present
}

#[allow(unused)]
pub(crate) struct FuncSignature {
    pub params: Vec<u8>,  // Parameter types (e.g., 0x7F for i32)
    pub returns: Vec<u8>, // Return types (e.g., 0x7F for i32)
}

/// `Func` represents a function body. Includes:
///     1. `locals`: a vector of (locals_count, locals_type) tuples
///     2. `body`: a vector of raw instructions (bytes)
///     3. `signature_index`: index into the types section
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Func {
    pub params: Vec<ValType>,   // Function parameter types
    pub locals: Vec<(u32, u8)>, // (locals_count, locals_type) pairs for local variables
    pub body: Vec<u8>,          // function body (instructions) as raw Wasm bytecode
    pub signature_index: u32,   // index into the types section
}

#[derive(Debug)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

impl ValType {
    pub fn default(&self) -> i32 {
        match self {
            ValType::I32 => 0,
            ValType::I64 => 0,
            ValType::F32 => 0,
            ValType::F64 => 0,
        }
    }
}

impl From<u8> for ValType {
    fn from(byte: u8) -> Self {
        match byte {
            0x7F => ValType::I32,
            0x7E => ValType::I64,
            0x7D => ValType::F32,
            0x7C => ValType::F64,
            _ => panic!("Unknown Wasm value type: 0x{:X}", byte),
        }
    }
}

pub(crate) struct Export {
    pub name: String,     // Exported function name
    pub kind: ExportKind, // The kind of export (function, memory, etc.)
    pub index: usize,     // Index in the corresponding section
}

pub(crate) enum ExportKind {
    Function,
    Memory,
    Table,
    Global,
}

#[allow(unused)]
pub(crate) struct Import {
    pub module: String,   // Module name
    pub name: String,     // Import name
    pub kind: ImportKind, // The kind of import (function, memory, etc.)
}
pub(crate) enum ImportKind {
    Function,
    Memory,
    Table,
    Global,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Memory {
    pub min: u32,         // Minimum pages (64KB each)
    pub max: Option<u32>, // Optional maximum pages
}

/// `OperandStack` stores the intermediate values during execution.
#[derive(Debug)]
pub struct OperandStack {
    stack: Vec<i32>,
}

impl OperandStack {
    /// Create a new `OperandStack`
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Check if the stack is empty
    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Push an operand onto the stack
    pub fn push(&mut self, value: i32) {
        self.stack.push(value);
    }

    /// Pop an operand from the stack
    pub fn pop(&mut self) -> Option<i32> {
        if self.is_empty() {
            return None;
        }

        self.stack.pop()
    }
}

/// `Context` is the execution context that stores:
///     1. the operand stack
///     2. local variables
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Context<'a> {
    pub locals: Vec<i32>,      // Local variables for the current function
    pub memory: Memory,        // Linear memory for WASI / runtime
    pub stack: OperandStack,   // The stack for calculations
    pub functions: &'a [Func], // Available functions in the module
}

impl<'a> Context<'a> {
    /// Create a new execution `Context`
    pub fn new(memory: Memory, functions: &'a [Func]) -> Self {
        Self {
            locals: Vec::new(),
            memory,
            stack: OperandStack::new(),
            functions,
        }
    }
}
