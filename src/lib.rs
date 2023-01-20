mod sys;

pub use sys::*;

#[repr(transparent)]
pub struct JumpBuf([usize; sys::JUMP_BUF_SIZE]);

impl JumpBuf {
    pub const fn new() -> JumpBuf {
        JumpBuf([0; sys::JUMP_BUF_SIZE])
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[allow(unreachable_code)]
    fn it_works() {
        let mut sjlj_buf = JumpBuf::new();

        let mut x = 42;
        if unsafe { setjmp(&mut sjlj_buf) } != 0 {
            // Volatile read to get around the return_twice issue
            // https://github.com/rust-lang/rfcs/issues/2625
            unsafe { std::ptr::read_volatile(&x as *const _) };
            debug_assert!(x == 13);
            return;
        }

        x = 13;
        debug_assert!(x == 13);
        unsafe { longjmp(&sjlj_buf, 1) }
        panic!("Should never reach here");
    }
}
