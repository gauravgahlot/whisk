use crate::*;

/// `OpCode` defines a (sub)set of supported WebAssembly instructions
enum OpCode {
    Call(u32),
    LocalGet(u32),
    LocalSet(u32),
    I32Constant(i32),
    I32Add,
    I32Mul,
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
        0x10 => {
            let (func_index, size) = leb128::decode(&bytes[1..]);
            (OpCode::Call(func_index as u32), 1 + size)
        }
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
        0x6C => (OpCode::I32Mul, 1),
        0x0F => (OpCode::Return, 1),
        0x0B => (OpCode::End, 1),
        op => (OpCode::Unimplemented(op), 1),
    }
}

pub(crate) fn execute_function(
    ctx: &mut types::Context,
    func: &types::Func,
    args: &[i32],
) -> Option<i32> {
    // ensure function parameters are initialized from arguments
    ctx.locals = func
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| args.get(i).copied().unwrap_or_else(|| ty.default())) // Use args if provided
        .chain(
            func.locals
                .iter()
                .flat_map(|(count, ty)| vec![default_value(*ty); *count as usize]),
        )
        .collect();

    let mut pc = 0; // program counter
    let mut call_stack: Vec<usize> = Vec::new(); // Stack for function call returns

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
            OpCode::I32Mul => {
                let x = ctx.stack.pop().unwrap();
                let y = ctx.stack.pop().unwrap();
                ctx.stack.push(x * y);
            }
            OpCode::Call(index) => {
                let called_func = &ctx.functions[index as usize];
                let arg_count = called_func.params.len();
                let mut args = vec![0; arg_count];

                // Extract arguments from stack
                for i in (0..arg_count).rev() {
                    args[i] = ctx.stack.pop().unwrap();
                }

                call_stack.push(pc); // Save return position
                let result = execute_function(ctx, called_func, &args);

                if let Some(value) = result {
                    ctx.stack.push(value); // Push return value onto stack
                }

                pc = func.body.len(); // Ensure function returns control properly
            }
            OpCode::Return => {
                if let Some(ret_addr) = call_stack.pop() {
                    pc = ret_addr; // return to caller
                } else {
                    let result = ctx.stack.pop();
                    return result; // return final value if no caller
                }
            }
            OpCode::End => {
                if let Some(ret_addr) = call_stack.pop() {
                    pc = ret_addr;
                } else {
                    let result = ctx.stack.pop();
                    return result;
                }
            }
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
