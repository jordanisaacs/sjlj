use crate::JumpBuf;

pub const JUMP_BUF_SIZE: usize = 8;

// http://git.musl-libc.org/cgit/musl/tree/src/setjmp/x86_64

/// Dynamically establishes the target to which control will later be transferred.
///
/// Saves various information about the calling environment in the [`crate::JumpBuf`] env.
/// Signal masks are not are not saved/restored. Returns 0 when called. When returned to
/// from [`crate::longjmp`] it returns the `ret` value passed in (always greater than 0)
///
/// Safety:
///
/// Protect against the [return twice](https://github.com/rust-lang/rfcs/issues/2625)
/// miscompilation. Use volatile reads/writes to get around it.
#[naked_function::naked]
pub unsafe extern "C" fn setjmp(env: &mut JumpBuf) -> u32 {
    asm!(
        "mov [rdi],    rbx",     // jmp_buf[0] = rbx
        "mov [rdi+8],  rbp",     // jmp_buf[1] = rbp
        "mov [rdi+16], r12",     // jmp_buf[2] = r12
        "mov [rdi+24], r13",     // jmp_buf[3] = r13
        "mov [rdi+32], r14",     // jmp_buf[4] = r14
        "mov [rdi+40], r15",     // jmp_buf[5] = r15
        "lea rdx,      [rsp+8]", // Get previous value of rsp, before call
        "mov [rdi+48], rdx",     // jmp_buf[6] = ^
        "mov rdx,      [rsp]",   // Get saved caller eip from top of stack
        "mov [rdi+56], rdx",     // jmp_buf[7] = ^
        "xor eax,      eax",     // Always return 0
        "ret",
    )
}

/// Performs transfer of execution to a location dynamically established by `crate::setjmp`.
///
/// Loads information from the [`crate::JumpBuf`] env. Returns the `ret` value to the caller of
/// [`crate::setjmp`]. If `ret` was mistakenly given as 0, it is incremented to 1.
///
/// Safety:
///
/// Can only jump over any frames that are "Plain Old Frames", frames that can be trivially
/// POF frames do not contain any pending destructors (live `Drop` objects) or `catch_unwind`
/// calls. (see c-unwind [RFC](https://github.com/rust-lang/rfcs/blob/master/text/2945-c-unwind-abi.md#plain-old-frames))
#[naked_function::naked]
pub unsafe extern "C" fn longjmp(env: &JumpBuf, ret: u32) -> ! {
    asm!(
        "xor eax, eax",      // set eax to 0
        "cmp esi, 1",        // CF = val ? 0 : 1
        "adc eax, esi",      // eax = val + !val ; These two lines add one to ret if equals 0
        "mov rbx, [rdi]",    // rbx = jmp_buf[0]
        "mov rbp, [rdi+8]",  // rbp = jmp_buf[1]
        "mov r12, [rdi+16]", // r12 = jmp_buf[2]
        "mov r13, [rdi+24]", // r13 = jmp_buf[4]
        "mov r14, [rdi+32]", // r14 = jmp_buf[5]
        "mov r15, [rdi+40]", // r15 = jmp_buf[6]
        "mov rsp, [rdi+48]", // rsp = jmp_buf[7] ; Value of rsp before setjmp call
        "jmp [rdi+56]",      // jmp to absolute address of jmp_buf[8] ; saved caller eip
    )
}
