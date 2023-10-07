use std::mem;

use crate::rt::ffi;

pub trait UnsafeConf {
    type T: Copy;

    const COMMANDS: Self::T;
}

impl UnsafeConf for () {
    type T = [ffi::ngx_command_t; 0];

    const COMMANDS: [ffi::ngx_command_t; 0] = [];
}

#[doc(hidden)]
#[macro_export]
macro_rules! const_concat {
    ($a:expr, $b:expr) => {
        $crate::conf::const_concat2($a, $b)
    };
    ($a:expr, $b:expr, $c:expr) => {
        $crate::conf::const_concat3($a, $b, $c)
    };
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        $crate::conf::const_concat4($a, $b, $c, $d)
    };
}

#[doc(hidden)]
pub const unsafe fn const_concat2<A: Copy, B: Copy, C: Copy>(a: A, b: B) -> C {
    #[repr(C)]
    struct Both<A, B>(A, B);

    union Transmute<A, B, C> {
        from: mem::ManuallyDrop<Both<A, B>>,
        to: mem::ManuallyDrop<C>,
    }

    mem::ManuallyDrop::into_inner(
        Transmute {
            from: mem::ManuallyDrop::new(Both(a, b)),
        }
        .to,
    )
}

#[doc(hidden)]
pub const unsafe fn const_concat3<A: Copy, B: Copy, C: Copy, D: Copy>(a: A, b: B, c: C) -> D {
    #[repr(C)]
    struct Tuple<A, B, C>(A, B, C);

    union Transmute<A, B, C, D> {
        from: mem::ManuallyDrop<Tuple<A, B, C>>,
        to: mem::ManuallyDrop<D>,
    }

    mem::ManuallyDrop::into_inner(
        Transmute {
            from: mem::ManuallyDrop::new(Tuple(a, b, c)),
        }
        .to,
    )
}

#[doc(hidden)]
pub const unsafe fn const_concat4<A: Copy, B: Copy, C: Copy, D: Copy, E: Copy>(
    a: A,
    b: B,
    c: C,
    d: D,
) -> E {
    #[repr(C)]
    struct Tuple<A, B, C, D>(A, B, C, D);

    union Transmute<A, B, C, D, E> {
        from: mem::ManuallyDrop<Tuple<A, B, C, D>>,
        to: mem::ManuallyDrop<E>,
    }

    mem::ManuallyDrop::into_inner(
        Transmute {
            from: mem::ManuallyDrop::new(Tuple(a, b, c, d)),
        }
        .to,
    )
}
