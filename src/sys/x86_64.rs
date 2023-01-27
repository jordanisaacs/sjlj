#[cfg(target_os = "linux")]
use super::{sigsetjmp_tail, SigJumpBuf};

use crate::JumpBuf;

pub(crate) const JUMP_BUF_SIZE: usize = 8;

// http://git.musl-libc.org/cgit/musl/tree/src/setjmp/x86_64

/// Dynamically establishes the target to which control will later be transferred.
///
/// Saves various information about the calling environment in the [`crate::SigJumpBuf`] env.
/// Signal masks can saved/restored if `save_mask` is set. Returns 0 when called. When returned to
/// from [`crate::siglongjmp`] it returns the `ret` value passed in (always greater than 0)
///
/// Safety:
///
/// Protect against the [return twice](https://github.com/rust-lang/rfcs/issues/2625)
/// miscompilation. Use volatile reads/writes to get around it.
#[naked_function::naked]
#[cfg(target_os = "linux")]
pub unsafe extern "C" fn sigsetjmp(env: &mut SigJumpBuf, save_mask: bool) -> u32 {
    // TODO: Use target_os cfg when naked_function merges cfg passthrough
    asm!(
        "test esi, esi",           // If save_mask if false then just do a normal setjmp (will skip
        "jz 1f",                   // sigsetjmp_tail)

        "pop QWORD PTR [rdi+64]",  // Pop caller saved eip in env.fl
        "mov [rdi+72], rbx",       // Save rbx in env.ss
        "mov rbx, rdi",            // rbx = env address (caller owned)

        "call {0}",                // Call setjmp

        "push QWORD PTR [64+rbx]", // Push env.fl (previous top of stack) onto top of stack
        "mov rdi, rbx",            // Put env as first argument
        "mov esi, eax",            // Set the setjmp ret value as second argument
        "mov rbx, [rbx+72]",       // Restore rbx
        "jmp {1}",

        "1:",
        "jmp {0}",
        sym setjmp,
        sym sigsetjmp_tail,
    )
}

/// Dynamically establishes the target to which control will later be transferred.
///
/// Saves various information about the calling environment in the [`crate::JumpBuf`] env.
/// Signal masks are not saved/restored. Returns 0 when called. When returned to
/// from [`crate::longjmp`] it returns the `ret` value passed in (always greater than 0)
///
/// Safety:
///
/// Protect against the [return twice](https://github.com/rust-lang/rfcs/issues/2625)
/// miscompilation. Use volatile reads/writes to get around it.
#[naked_function::naked]
pub unsafe extern "C" fn setjmp(env: &mut JumpBuf) -> u32 {
    asm!(
        "mov [rdi],    rbx",     // Store caller saved registers
        "mov [rdi+8],  rbp",     // ^
        "mov [rdi+16], r12",     // ^
        "mov [rdi+24], r13",     // ^
        "mov [rdi+32], r14",     // ^
        "mov [rdi+40], r15",     // ^
        "lea rdx,      [rsp+8]", // go one value up (as if setjmp wasn't called)
        "mov [rdi+48], rdx",     // Store the new rsp pointer in env[7]
        "mov rdx,      [rsp]",   // go one value up (as if setjmp wasn't called)
        "mov [rdi+56], rdx",     // Store the address we will resume at in env[8]
        "xor eax,      eax",     // Always return 0
        "ret",
    )
}

/// Performs transfer of execution to a location dynamically established by [`crate::setjmp`].
///
/// Loads information from the [`crate::JumpBuf`] env. Returns the `ret` value to the caller of
/// [`crate::setjmp`]. If `ret` was mistakenly given as 0, it is incremented to 1.
///
/// Safety:
///
/// Can only jump over any frames that are "Plain Old Frames," aka frames that can be trivially
/// deallocated. POF frames do not contain any pending destructors (live `Drop` objects) or
/// `catch_unwind` calls. (see c-unwind [RFC](https://github.com/rust-lang/rfcs/blob/master/text/2945-c-unwind-abi.md#plain-old-frames))
#[naked_function::naked]
pub unsafe extern "C" fn longjmp(env: &JumpBuf, ret: u32) -> ! {
    asm!(
        "xor eax, eax",      // set eax to 0
        "cmp esi, 1",        // CF = val ? 0 : 1
        "adc eax, esi",      // eax = val + !val ; These two lines add one to ret if equals 0
        "mov rbx, [rdi]",    // Load in caller saved registers
        "mov rbp, [rdi+8]",  // ^
        "mov r12, [rdi+16]", // ^
        "mov r13, [rdi+24]", // ^
        "mov r14, [rdi+32]", // ^
        "mov r15, [rdi+40]", // ^
        "mov rsp, [rdi+48]", // Value of rsp before setjmp call
        "jmp [rdi+56]",      // goto saved address without altering rsp
    )
}
