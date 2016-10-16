#![feature(trace_macros)]

#[macro_use]
extern crate enum_primitive;
extern crate libc;

use std::io;

fn os_errno() -> usize {
    return io::Error::last_os_error().raw_os_error().unwrap_or(0) as usize;
}

pub mod signals {
    use super::libc;
    use std::mem;
    use super::os_errno;

    enum_from_primitive!{
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub enum Signal {
        None = 0,
        Hup,
        Int,
        Quit,
        Ill,
        Trap,
        Abrt,
        Bus,
        Fpe,
        Kill,
        Usr1,
        Segv,
        Usr2,
        Pipe,
        Alrm,
        Term,
        StkFlt,
        Chld,
        Cont,
        Stop,
        Tstp,
        Ttin,
        Ttou,
        Urg,
        XCpu,
        Xfsz,
        Vtalrm,
        Prof,
        Winch,
        Io,
        Pwr,
        Sys
    }
    }

    impl Signal {
        pub fn raise(self) -> Result<(), usize> {
            match unsafe { raise(self as libc::c_int) } {
                0 => Result::Ok(()),
                _ => Result::Err(os_errno())
            }
        }

        pub fn kill(self, pid: libc::pid_t) -> Result<(), usize> {
            match unsafe { kill(pid, self as libc::c_int) } {
                0 => Result::Ok(()),
                _ => Result::Err(os_errno())
            }
        }

        pub unsafe fn handle(self, handler: Box<FnMut(Signal)>) -> Result<(), usize> {
            match signal(self as libc::c_int, mem::transmute(glue::rust_signal_handler as unsafe extern "C" fn(_))) {
                - 1 => Result::Err(os_errno()),
                _ => {
                    glue::set_handler(self, handler);
                    Result::Ok(())
                }
            }
        }
    }

    mod glue {
        use super::Signal;
        use enum_primitive::FromPrimitive;
        use super::libc;
        use std::mem;

        #[derive(Clone, Copy, Debug)]
        struct FnPtr {
            foo: usize,
            bar: usize
        }

        static mut handlers: [FnPtr; 18] = [
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
            FnPtr { foo: 0, bar: 0 },
        ];

        pub unsafe fn set_handler(sig: Signal, f: Box<FnMut(Signal)>) {
            handlers[sig as usize] = mem::transmute(f);
        }

        #[allow(dead_code, unused_variables)]
        fn null_handler(s: Signal) {}

        pub unsafe extern "C" fn rust_signal_handler(sig: libc::c_int) {
            let f: *mut FnMut(Signal) = mem::transmute(handlers[sig as usize]);
            let p: FnPtr = mem::transmute(f);
            if p.foo != 0 && p.bar != 0 {
                match Signal::from_i32(sig) {
                    Some(s) => (*f)(s),
                    None => panic!("Unknown signal {}", sig)
                }
            }
        }
    }

    extern "C" {
        fn raise(sig: libc::c_int) -> libc::c_int;
        fn signal(sig: libc::c_int, handler: *const libc::c_void) -> libc::c_int;
        fn kill(pid: libc::pid_t, sig: libc::c_int) -> libc::c_int;
    }
}
