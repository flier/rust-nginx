mod array;
mod buf;
mod conf;
mod cycle;
mod err;
mod log;
mod module;
mod pool;
mod status;
mod str;

pub use self::array::{Array, ArrayRef};
pub use self::buf::{Buf, BufRef};
pub use self::conf::{Conf, ConfFile, ConfFileRef, ConfRef, NGX_CONF_ERROR, NGX_CONF_OK};
pub use self::cycle::{Cycle, CycleRef};
pub use self::log::{Level as LogLevel, Log, LogRef};
pub use self::module::Type as ModuleType;
pub use self::pool::{Pool, PoolRef};
pub use self::status::Code;
pub use self::str::Str;

pub fn fake_drop<T>(_: *mut T) {
    unreachable!()
}
