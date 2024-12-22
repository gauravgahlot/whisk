use super::executor::Context;

pub(crate) fn fd_write(ctx: &mut Context, args: &[i32]) -> Option<i32> {
    let fd = args[0]; // File descriptor
    let iovs_ptr = args[1] as usize; // Pointer to iovecs
    let iovs_len = args[2] as usize; // Number of iovecs
    let nwritten_ptr = args[3] as usize; // Pointer to write result

    if fd != 1 && fd != 2 {
        return Some(52); // __WASI_ERRNO_BADF
    }

    // Decode and process the iovecs
    let mut total_bytes_written = 0;
    for i in 0..iovs_len {
        let base = iovs_ptr + i * 8;
        let ptr = u32::from_le_bytes(ctx.memory[base..base + 4].try_into().unwrap()) as usize;
        let len = u32::from_le_bytes(ctx.memory[base + 4..base + 8].try_into().unwrap()) as usize;

        let data = &ctx.memory[ptr..ptr + len];
        print!("{}", String::from_utf8_lossy(data));
        total_bytes_written += len;
    }

    // Write the number of bytes written back to memory
    ctx.memory[nwritten_ptr..nwritten_ptr + 4]
        .copy_from_slice(&(total_bytes_written as u32).to_le_bytes());

    Some(0) // __WASI_ERRNO_SUCCESS
}
