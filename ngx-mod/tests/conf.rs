use std::mem;

use ngx_mod::{
    rt::{
        core::{ArrayRef, Str},
        ngx_str,
    },
    Conf,
};

use foreign_types::ForeignTypeRef;

#[derive(Clone, Debug, Conf, Default)]
#[conf(http::upstream)]
pub struct MainConf {
    #[directive(args(0, 1))]
    pub max: isize,
}

#[derive(Clone, Debug, Conf)]
#[conf(http::main | http::server, default = zeroed)]
pub struct SrvConf {
    #[directive(args(0, 1))]
    pub max: isize,
    #[directive(args(0, 1))]
    pub min: isize,
}

#[derive(Clone, Debug, Conf)]
#[conf(http::main | http::server | http::location, default = unset)]
pub struct LocConf {
    pub u32: u32,
    pub usize: usize,
    pub isize: isize,
    pub cptr: *const u8,
    pub ptr: *mut u8,
    pub opt: Option<usize>,
    pub str: Str,
    pub ngx_str: <Str as ForeignTypeRef>::CType,
    pub ngx_array: <ArrayRef<u8> as ForeignTypeRef>::CType,
}

#[test]
fn main_conf() {
    let c = MainConf::default();

    assert_eq!(c.max, 0);
}

#[test]
fn srv_conf() {
    let c = SrvConf::default();

    assert_eq!(c.max, 0);
    assert_eq!(c.min, 0);
}

#[test]
fn loc_conf() {
    let c = LocConf::default();

    assert_eq!(c.u32, u32::MAX);
    assert_eq!(c.usize, usize::MAX);
    assert_eq!(c.isize, usize::MAX as isize);
    assert_eq!(c.cptr, usize::MAX as *const _);
    assert_eq!(c.ptr, usize::MAX as *mut _);
    assert_eq!(c.opt, None);
    assert_eq!(c.str, Str::null());
    assert_eq!(c.ngx_str, ngx_str!());
    assert_eq!(c.ngx_array, unsafe { mem::zeroed() });
}
