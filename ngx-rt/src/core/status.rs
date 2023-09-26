use crate::ffi;

#[repr(isize)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Code {
    #[default]
    Ok = ffi::NGX_OK as isize,
    Error = ffi::NGX_ERROR as isize,
    Again = ffi::NGX_AGAIN as isize,
    Busy = ffi::NGX_BUSY as isize,
    Done = ffi::NGX_DONE as isize,
    Declined = ffi::NGX_DECLINED as isize,
    Abort = ffi::NGX_ABORT as isize,
}
