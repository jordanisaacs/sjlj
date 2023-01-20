use linux_raw_sys::general::{__NR_rt_sigprocmask, sigset_t, SIG_SETMASK};

use super::{longjmp, JUMP_BUF_SIZE};

#[repr(C)]
pub struct SigJumpBuf {
    jmp_buf: [usize; JUMP_BUF_SIZE],
    fl: usize,
    rbx: usize,
    ss: [u8; Self::SS_SIZE],
}

impl SigJumpBuf {
    const SS_SIZE: usize = std::mem::size_of::<sigset_t>();

    pub const fn new() -> SigJumpBuf {
        SigJumpBuf {
            jmp_buf: [0; JUMP_BUF_SIZE],
            fl: 0,
            rbx: 0,
            ss: [0; Self::SS_SIZE],
        }
    }
}

/// If returning from setjmp, then store the signal mask in `ss`.
/// If returning from longjmp, then set the signal mask to the value in `ss`
/// Only called from sigsetjmp inline asm
pub(crate) unsafe extern "C" fn sigsetjmp_tail(env: &mut SigJumpBuf, ret: u32) -> u32 {
    let p = &mut env.ss as *mut _ as usize;
    let set = if ret != 0 { p } else { 0 };
    let oldset = if ret != 0 { 0 } else { p };

    let r = sc::syscall4(
        __NR_rt_sigprocmask as usize,
        SIG_SETMASK as usize,
        set,
        oldset,
        SigJumpBuf::SS_SIZE,
    );
    debug_assert!(r == 0);

    ret
}

/// Performs transfer of execution to a location dynamically established by [`crate::sigsetjmp`].
///
/// Does the same thing as [`crate::longjmp`] but takes a [`crate::SigJumpBuf`] that can
/// restore signal masks of restoring signal masks (if set by [`crate::sigsetjmp`]).
/// This makes is *safer* to use in signal handlers.
#[inline]
pub unsafe fn siglongjmp(env: &SigJumpBuf, ret: u32) -> ! {
    longjmp(&std::mem::transmute(env.jmp_buf), ret);
}
