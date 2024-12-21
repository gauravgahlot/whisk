use super::leb128;

/// `OperandStack` stores the intermediate values during execution.
struct OperandStack {
    stack: Vec<i32>,
}

impl OperandStack {
    /// Create a new `OperandStack`
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Check if the stack is empty
    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Push an operand onto the stack
    fn push(&mut self, value: i32) {
        self.stack.push(value);
    }

    /// Pop an operand from the stack
    fn pop(&mut self) -> Option<i32> {
        if self.is_empty() {
            return None;
        }

        self.stack.pop()
    }
}

/// `Func` represents a function body. Includes:
///     1. `locals`: a vector of (locals_count, locals_type) tuples
///     2. `body`: a vector of raw instructions (bytes)
pub(crate) struct Func {
    locals: Vec<(u32, u8)>, // (locals_count, locals_type)
    body: Vec<u8>,          // raw instructions
}

impl Func {
    /// Create a new `Func` with given `locals` and instructions (`body`)
    pub(crate) fn new(locals: Vec<(u32, u8)>, body: Vec<u8>) -> Self {
        Self { locals, body }
    }
}

/// `Context` is the execution context that stores:
///     1. the operand stack
///     2. local variables
pub(crate) struct Context {
    stack: OperandStack,
    locals: Vec<i32>,
}

impl Context {
    /// Create a new execution `Context`
    pub(crate) fn new() -> Self {
        Self {
            locals: Vec::new(),
            stack: OperandStack::new(),
        }
    }
}

/// `OpCode` defines a (sub)set of supported WebAssembly instructions
enum OpCode {
    LocalGet(u32),
    LocalSet(u32),
    I32Constant(i32),
    I32Add,
    Return,
    End,
    Unimplemented(u8),
}

/// Instructions like `local.get` and `local.set` are followed by a local index
/// (which is LEB128 encoded).
/// `decode_instruction` decodes the instruction, and returns the
/// `(OpCode, index)`
fn decode_instruction(bytes: &[u8]) -> (OpCode, usize) {
    match bytes[0] {
        0x20 => {
            let (index, size) = leb128::decode(&bytes[1..]);
            (OpCode::LocalGet(index as u32), 1 + size)
        }
        0x21 => {
            let (index, size) = leb128::decode(&bytes[1..]);
            (OpCode::LocalSet(index as u32), 1 + size)
        }
        0x41 => {
            let (value, size) = leb128::decode(&bytes[1..]);
            (OpCode::I32Constant(value as i32), 1 + size)
        }
        0x6A => (OpCode::I32Add, 1),
        0x0F => (OpCode::Return, 1),
        0x0B => (OpCode::End, 1),
        op => (OpCode::Unimplemented(op), 1),
    }
}

pub(crate) fn execute_function(ctx: &mut Context, func: &Func) -> Option<i32> {
    // initialize locals with default values
    ctx.locals = func
        .locals
        .iter()
        .flat_map(|(count, ty)| vec![default_value(*ty); *count as usize])
        .collect();

    let mut pc = 0; // program counter
    while pc < func.body.len() {
        let (op_code, size) = decode_instruction(&func.body[pc..]);
        pc += size;

        match op_code {
            OpCode::LocalGet(index) => {
                let value = ctx.locals[index as usize];
                ctx.stack.push(value);
            }
            OpCode::LocalSet(index) => {
                let value = ctx.stack.pop().unwrap();
                ctx.locals[index as usize] = value;
            }
            OpCode::I32Constant(value) => ctx.stack.push(value),
            OpCode::I32Add => {
                let x = ctx.stack.pop().unwrap();
                let y = ctx.stack.pop().unwrap();
                ctx.stack.push(x + y);
            }
            OpCode::Return => {
                return ctx.stack.pop();
            }
            OpCode::End => break,
            OpCode::Unimplemented(op) => {
                panic!("unimplemented opcode: 0x{:02x}", op);
            }
        }
    }

    None
}

fn default_value(ty: u8) -> i32 {
    match ty {
        0x7F => 0, // i32
        0x7E => 0, // i64
        _ => panic!("unsupported local type: 0x{:02x}", ty),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_function() {
        // local.get 0   ;; 0x20 0x00
        // local.get 1   ;; 0x20 0x01
        // i32.add       ;; 0x6A
        // local.set 2   ;; 0x21 0x02
        // local.get 2   ;; 0x20 0x02
        // end           ;; 0x0B

        let func = Func {
            // three locals of type i32
            locals: vec![(1, 0x7F), (1, 0x7F), (1, 0x7F)],
            body: vec![
                0x41, 0x0A, // i32.const 10
                0x21, 0x00, // local.set 0
                0x41, 0x14, // i32.const 20
                0x21, 0x01, // local.set 1
                0x20, 0x00, // local.get 0
                0x20, 0x01, // local.get 1
                0x6A, // i32.add
                0x21, 0x02, // local.set 2
                0x20, 0x02, // local.get 2
                0x0B, // end
            ],
        };

        // create an execution context
        let mut ctx = Context::new();

        let result = execute_function(&mut ctx, &func);
        assert_eq!(result, None);
        assert_eq!(ctx.stack.pop(), Some(30));
        assert_eq!(ctx.locals[2], 30);
    }

    #[test]
    fn test_function_return() {
        let func = Func {
            locals: vec![],
            // i32.const 10, i32.const 20, i32.add, return
            body: vec![0x41, 0x0A, 0x41, 0x14, 0x6A, 0x0F],
        };

        let mut ctx = Context::new();
        let result = execute_function(&mut ctx, &func);
        assert_eq!(result, Some(30));
        assert!(ctx.stack.is_empty());
    }

    #[test]
    fn test_function_no_return() {
        let mut ctx = Context::new();
        let func = Func {
            locals: vec![],
            // i32.const 42, end
            body: vec![0x41, 0x2A, 0x0B],
        };

        let result = execute_function(&mut ctx, &func);
        assert_eq!(result, None);
    }
}
