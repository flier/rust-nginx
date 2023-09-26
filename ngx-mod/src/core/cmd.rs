use std::ffi::{c_char, c_void};

use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignTypeRef};
use ngx_rt::core::{fake_drop, ConfRef, Str, NGX_CONF_ERROR, NGX_CONF_OK};

use crate::ffi;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Type: u32 {
        /// Directive takes no arguments.
        const NO_ARGS   = ffi::NGX_CONF_NOARGS;
        /// Directive takes 1 arguments.
        const TAKE1     = ffi::NGX_CONF_TAKE1;
        /// Directive takes 2 arguments.
        const TAKE2     = ffi::NGX_CONF_TAKE2;
        /// Directive takes 3 arguments.
        const TAKE3     = ffi::NGX_CONF_TAKE3;
        /// Directive takes 4 arguments.
        const TAKE4     = ffi::NGX_CONF_TAKE4;
        /// Directive takes 5 arguments.
        const TAKE5     = ffi::NGX_CONF_TAKE5;
        /// Directive takes 6 arguments.
        const TAKE6     = ffi::NGX_CONF_TAKE6;
        /// Directive takes 7 arguments.
        const TAKE7     = ffi::NGX_CONF_TAKE7;

        /// Directive may take one or two arguments.
        const TAKE12    = ffi::NGX_CONF_TAKE12;
        /// Directive may take one or three arguments.
        const TAKE13    = ffi::NGX_CONF_TAKE13;
        /// Directive may take two or three arguments.
        const TAKE23    = ffi::NGX_CONF_TAKE23;
        /// Directive may take one, two or three arguments.
        const TAKE123   = ffi::NGX_CONF_TAKE123;
        /// Directive may take one, two, three or four arguments.
        const TAKE1234  = ffi::NGX_CONF_TAKE1234;

        const ARGS_NUMBER   = ffi::NGX_CONF_ARGS_NUMBER;

        /// Directive is a block, that is, it can contain other directives within its opening and closing braces,
        /// or even implement its own parser to handle contents inside.
        const BLOCK         = ffi::NGX_CONF_BLOCK;
        /// Directive takes a boolean value, either on or off.
        const FLAG          = ffi::NGX_CONF_FLAG;
        /// Directive is a block, or takes a boolean value
        const ANY           = ffi::NGX_CONF_ANY;

        /// Directive takes one or more arguments.
        const ONE_MORE      = ffi::NGX_CONF_1MORE;
        /// Directive takes two or more arguments.
        const TWO_MORE      = ffi::NGX_CONF_2MORE;

        /// Used by modules that don't create a hierarchy of contexts and only have one global configuration.
        const DIRECT_CONF   = ffi::NGX_DIRECT_CONF;
        /// In the top level context.
        const MAIN_CONF     = ffi::NGX_MAIN_CONF;
        const ANY_CONF      = ffi::NGX_ANY_CONF;
    }
}

impl Type {
    pub const MAX_ARGS: usize = ffi::NGX_CONF_MAX_ARGS as usize;
}

foreign_type! {
    pub unsafe type Cmd: Send {
        type CType = ffi::ngx_command_t;

        fn drop = fake_drop::<ffi::ngx_command_t>;
    }
}

impl CmdRef {
    pub fn name(&self) -> &str {
        unsafe { Str::from_raw(self.as_raw().name).to_str().expect("name") }
    }

    pub fn ty(&self) -> Type {
        unsafe { Type::from_bits_truncate(self.as_raw().type_ as u32) }
    }

    unsafe fn as_raw(&self) -> &ffi::ngx_command_t {
        &*self.as_ptr()
    }
}

pub trait UnsafeSet {
    unsafe extern "C" fn set(
        cf: *mut ffi::ngx_conf_t,
        cmd: *mut ffi::ngx_command_t,
        conf: *mut c_void,
    ) -> *mut c_char;
}

impl<T: Set> UnsafeSet for T {
    unsafe extern "C" fn set(
        cf: *mut ffi::ngx_conf_t,
        cmd: *mut ffi::ngx_command_t,
        conf: *mut c_void,
    ) -> *mut c_char {
        T::set(
            ConfRef::from_ptr(cf),
            CmdRef::from_ptr(cmd),
            &mut *conf.cast(),
        )
        .map_or(NGX_CONF_ERROR, |_| NGX_CONF_OK)
    }
}

pub trait Set {
    type Error;
    type Conf;

    fn set(cf: &ConfRef, cmd: &CmdRef, conf: &mut Self::Conf) -> Result<(), Self::Error> {
        Ok(())
    }
}
