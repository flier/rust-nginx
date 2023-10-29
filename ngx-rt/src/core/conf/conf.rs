use std::ffi::CString;
#[cfg(target_family = "unix")]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{null_mut, NonNull};

use foreign_types::{foreign_type, ForeignTypeRef};

use crate::core::Cmds;
use crate::AsRawMut;
use crate::{
    box_copy, box_drop,
    core::{ArrayRef, CmdRef, CycleRef, ModuleType, PoolRef, Str},
    ffi, http, native_setter, AsRawRef, AsResult, Error,
};

foreign_type! {
    pub unsafe type Conf: Send {
        type CType = ffi::ngx_conf_t;

        fn drop = box_drop::<ffi::ngx_conf_t>;
        fn clone = box_copy::<ffi::ngx_conf_t>;
    }
}

impl ConfRef {
    property! {
        cycle: &CycleRef;
        pool: &PoolRef;
        temp_pool: &PoolRef;
        name: &CStr;
        args: &ArrayRef<Str>;
    }

    pub fn as_http_context(&self) -> Option<&http::ConfContextRef> {
        if self.module_type() == ModuleType::Http {
            unsafe {
                NonNull::new(self.as_raw().ctx)
                    .map(|p| http::ConfContextRef::from_ptr(p.cast().as_ptr()))
            }
        } else {
            None
        }
    }

    pub fn module_type(&self) -> ModuleType {
        ModuleType::from(unsafe { self.as_raw().module_type as u32 })
    }

    pub fn parse_block<T: ConfExt>(&self, c: &mut T) -> Result<(), Error> {
        let mut cf = self.to_owned();

        unsafe {
            cf.as_raw_mut().handler = Some(parse_block::<T>);
            cf.as_raw_mut().handler_conf = c as *mut _ as *mut _;

            ffi::ngx_conf_parse(cf.as_ptr(), null_mut()).ok()
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, filename: P) -> Result<(), Error> {
        let p = filename.as_ref();
        let s = CString::new(p.as_os_str().as_bytes())?;

        unsafe { ffi::ngx_conf_parse(self.as_ptr(), s.as_ptr() as *mut _).ok() }
    }
}

#[native_setter(log = cf)]
unsafe fn parse_block<T>(cf: &ConfRef, _dummy: Option<&CmdRef>, conf: &mut T) -> Result<(), Error>
where
    T: ConfExt,
{
    let name = cf.args().get(0).unwrap();

    if cf.args().len() != 2 {
        return Err(Error::ConfigError(CString::new(format!(
            "invalid number of arguments in directive `{}`",
            name,
        ))?));
    }

    for cmd in <T as ConfExt>::commands() {
        if cmd.name() != name {
            continue;
        }

        unsafe {
            return if let Some(f) = cmd.as_raw().set {
                f(cf.as_ptr(), cmd.as_ptr(), conf as *mut _ as *mut _).ok()
            } else {
                Err(Error::ConfigError(CString::new(format!(
                    "directive `{}` missing setter",
                    name
                ))?))
            };
        }
    }

    Err(Error::ConfigError(CString::new(format!(
        "unknown directive `{}` in `otel_exporter`",
        name
    ))?))
}

pub trait ConfExt: UnsafeConf {
    fn commands() -> Cmds<'static>;
}

impl ConfExt for () {
    fn commands() -> Cmds<'static> {
        Cmds::from(&[][..])
    }
}

pub trait UnsafeConf {
    type Commands: Copy;

    const COMMANDS: Self::Commands;
}

impl UnsafeConf for () {
    type Commands = [ffi::ngx_command_t; 0];

    const COMMANDS: Self::Commands = [];
}
