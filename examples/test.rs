use linux_raw_sys::general::{
    __NR_kill, __NR_rt_sigaction, __NR_rt_sigprocmask, __NR_rt_sigreturn, sigaction, sigset_t,
    SA_RESTORER, SIGUSR1,
};
// use nix::libc::SIGINT;
// use nix::sys::signal::{SaFlags, SigAction, SigHandler};
// use nix::sys::{signal, signalfd::SigSet};
use sjlj::{siglongjmp, sigsetjmp, SigJumpBuf};

static mut RETURN_HERE: SigJumpBuf = SigJumpBuf::new();
const RET: u32 = 2;
const MOCK_SIGNAL_AT: usize = 3;

#[inline]
fn return_early() -> ! {
    unsafe { siglongjmp(&RETURN_HERE, RET) };
}

extern "C" fn sigrestore() {
    debug_assert!(unsafe { sc::syscall0(__NR_rt_sigreturn as usize) } == 0);
}

fn register_signal_handler() {
    unsafe {
        let mut x: sigaction = std::mem::zeroed();
        x.sa_handler = Some(handle_signals);
        x.sa_mask = 0;
        x.sa_flags = SA_RESTORER as u64;
        x.sa_restorer = Some(sigrestore);
        let r = sc::syscall4(
            __NR_rt_sigaction as usize,
            SIGUSR1 as usize,
            &x as *const _ as usize,
            0,
            std::mem::size_of::<sigset_t>(),
        );
        debug_assert!(r == 0);
    }
}

extern "C" fn handle_signals(_: i32) {
    return_early();
}

fn print_depth(depth: usize) {
    for _ in 0..depth {
        print!("#");
    }
    println!();
}

fn dive(depth: usize, max_depth: usize) {
    print_depth(depth);

    if depth >= max_depth || depth > MOCK_SIGNAL_AT {
        panic!("Unreachable");
    } else {
        if depth == MOCK_SIGNAL_AT {
            unsafe {
                sc::syscall2(__NR_kill as usize, 0, SIGUSR1 as usize);
            }
        }
        dive(depth + 1, max_depth);
    }

    print_depth(depth);
}

const SIZE: usize = std::mem::size_of::<sigset_t>();
fn sigmask() -> [u8; SIZE] {
    unsafe {
        let mut s = [0; SIZE];
        let p = &mut s as *mut _ as usize;

        let r = sc::syscall4(__NR_rt_sigprocmask as usize, 0, 0, p, SIZE);
        debug_assert!(r == 0);

        s
    }
}

fn main() {
    register_signal_handler();

    let start = sigmask();
    let rc = unsafe { sigsetjmp(&mut RETURN_HERE, true) };
    if rc == 0 {
        dive(0, 10);
    } else {
        debug_assert!(rc == RET);
        let end = sigmask();
        debug_assert!(start == end);
        println!("early return");
        return;
    }
    panic!("Unreachable");
}
