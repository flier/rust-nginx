#![crate_type = "dylib"]

use std::ffi::c_char;

use ngx_mod::{
    ffi::{ngx_command_t, ngx_module_t},
    http,
    rt::ngx_str,
    Module,
};

#[derive(Module)]
#[module(name = "ngx_http_upstream_custom_module")]
struct HttpUpstreamCustomModule;

impl Module for HttpUpstreamCustomModule {}

impl http::Module for HttpUpstreamCustomModule {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ();
}

#[no_mangle]
pub static mut ngx_modules: [*const ngx_module_t; 2] = [
    unsafe { &ngx_http_upstream_custom_module as *const ngx_module_t },
    std::ptr::null(),
];

#[no_mangle]
pub static mut ngx_module_names: [*const c_char; 2] = [
    "ngx_http_upstream_custom_module\0".as_ptr() as *const i8,
    std::ptr::null(),
];

#[no_mangle]
pub static mut ngx_module_order: [*const c_char; 1] = [std::ptr::null()];

#[no_mangle]
static mut ngx_http_upstream_custom_commands: [ngx_command_t; 1] = [
    // ngx_command_t {
    //     name: ngx_string!("custom"),
    //     type_: (NGX_HTTP_UPS_CONF | NGX_CONF_NOARGS | NGX_CONF_TAKE1) as ngx_uint_t,
    //     set: Some(ngx_http_upstream_commands_set_custom),
    //     conf: NGX_RS_HTTP_SRV_CONF_OFFSET,
    //     offset: 0,
    //     post: std::ptr::null_mut(),
    // },
    ngx_command_t {
        name: ngx_str!(),
        type_: 0,
        set: None,
        conf: 0,
        offset: 0,
        post: ::std::ptr::null_mut(),
    },
];
