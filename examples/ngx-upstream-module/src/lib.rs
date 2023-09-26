#![crate_type = "dylib"]

use std::ffi::c_char;

use merge::Merge as AutoMerge;

use ngx_mod::{
    core::{CmdRef, Setter},
    ffi::{self, ngx_command_t, ngx_module_t},
    http,
    rt::{
        core::{ConfRef, LogLevel},
        ngx_str,
    },
    Merge, Module,
};

#[derive(Module)]
#[module(name = "ngx_http_upstream_custom_module", type = http)]
struct HttpUpstreamCustomModule;

impl Module for HttpUpstreamCustomModule {}

impl http::Module for HttpUpstreamCustomModule {
    type Error = ();
    type MainConf = ();
    type SrvConf = SrvConfig;
    type LocConf = ();
}

#[derive(Clone, Debug, AutoMerge)]
struct SrvConfig {
    #[merge(strategy = merge::num::overwrite_zero)]
    max: u32,
    original_init_upstream: ffi::ngx_http_upstream_init_pt,
    original_init_peer: ffi::ngx_http_upstream_init_peer_pt,
}

impl Setter for SrvConfig {
    type Error = ();
    type Conf = SrvConfig;

    fn set(cf: &ConfRef, cmd: &CmdRef, conf: &mut Self::Conf) -> Result<(), Self::Error> {
        cf.log().http().debug("custom init upstream");

        if cf.args().len() == 2 {
            let s = cf.args().get(1).unwrap().to_str().expect("max");
            match s.parse() {
                Ok(n) => {
                    conf.max = n;
                }
                Err(err) => {
                    cf.emerg(format!(
                        "invalid value `{}` in `{}` directive, {}",
                        s,
                        cmd.name(),
                        err
                    ));

                    return Err(());
                }
            }
        }

        Ok(())
    }
}

impl Default for SrvConfig {
    fn default() -> Self {
        SrvConfig {
            max: u32::MAX,
            original_init_upstream: None,
            original_init_peer: None,
        }
    }
}

impl Merge for SrvConfig {
    type Error = ();

    fn merge(&mut self, prev: &SrvConfig) -> Result<(), ()> {
        merge::Merge::merge(self, prev.clone());

        Ok(())
    }
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
