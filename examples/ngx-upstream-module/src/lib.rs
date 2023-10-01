#![crate_type = "dylib"]

use std::{
    ffi::{c_char, c_void},
    ptr::NonNull,
};

use foreign_types::ForeignTypeRef;
use merge::Merge as AutoMerge;

use ngx_mod::{
    core::Setter,
    ffi::{self, ngx_command_t, ngx_module_t},
    http,
    rt::{
        core::{
            conf::{self, Unset},
            CmdRef, ConfRef, ConnRef, ModuleRef,
        },
        event::PeerConnRef,
        http::{upstream, RequestRef},
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

    fn create_srv_conf(cf: &ConfRef) -> Option<&mut Self::SrvConf> {
        if let Some(p) = cf.pool().allocate(Self::SrvConf::default()) {
            p.max = conf::unset();

            Some(p)
        } else {
            cf.emerg("could not allocate memory for config, out of memory");

            None
        }
    }
}

#[derive(Clone, Debug, AutoMerge)]
struct SrvConfig {
    #[merge(strategy = merge::num::overwrite_zero)]
    max: u32,
    original_init_upstream: ffi::ngx_http_upstream_init_pt,
    original_init_peer: ffi::ngx_http_upstream_init_peer_pt,
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

impl Setter for SrvConfig {
    type Error = ();
    type Conf = SrvConfig;

    fn set(cf: &ConfRef, cmd: &CmdRef, conf: &mut Self::Conf) -> Result<(), Self::Error> {
        cf.log().http().debug("custom init upstream");

        if cf.args().len() == 2 {
            let s = cf.args().get(1).unwrap().to_str().expect("max");
            match s.parse() {
                Ok(n) => {
                    if n > 0 {
                        conf.max = n;
                    } else {
                        cf.emerg(format!(
                            "invalid value `{}` in `{}` directive, {}",
                            s,
                            cmd.name().unwrap(),
                            "max must be greater than 0"
                        ));

                        return Err(());
                    }
                }
                Err(err) => {
                    cf.emerg(format!(
                        "invalid value `{}` in `{}` directive, {}",
                        s,
                        cmd.name().unwrap(),
                        err
                    ));

                    return Err(());
                }
            }
        }

        let uscf = unsafe {
            cf.as_http_context()
                .expect("ctx")
                .srv_conf::<upstream::SrvConf>(ffi::ngx_http_upstream_module.ctx_index)
                .expect("conf")
        };

        conf.original_init_upstream = uscf
            .peer()
            .init_upstream
            .or(Some(ffi::ngx_http_upstream_init_round_robin));

        uscf.peer_mut().init_upstream = Some(ngx_http_upstream_init_custom);

        Ok(())
    }
}

#[no_mangle]
unsafe extern "C" fn ngx_http_upstream_init_custom(
    cf: *mut ffi::ngx_conf_t,
    us: *mut ffi::ngx_http_upstream_srv_conf_t,
) -> ffi::ngx_int_t {
    let cf = ConfRef::from_ptr(cf);
    let us = upstream::SrvConfRef::from_ptr_mut(us);
    let log = cf.log().http();

    log.debug("custom init upstream");

    let original_init_upstream = if let Some(hccf) = us.srv_conf_mut::<SrvConfig>(
        ModuleRef::from_ptr(&mut ngx_http_upstream_custom_module as *mut _),
    ) {
        hccf.max.get_or_set(100);
        hccf.original_init_upstream
    } else {
        cf.emerg("no upstream srv_conf");

        return ffi::NGX_ERROR as isize;
    };

    if let Some(f) = original_init_upstream {
        if f(cf.as_ptr(), us.as_ptr()) != ffi::NGX_OK as isize {
            log.debug("failed calling init_upstream");

            return ffi::NGX_ERROR as isize;
        }
    }

    let original_init_peer = us.peer_mut().init.replace(http_upstream_init_custom_peer);

    if let Some(hccf) = us.srv_conf_mut::<SrvConfig>(ModuleRef::from_ptr(
        &mut ngx_http_upstream_custom_module as *mut _,
    )) {
        hccf.original_init_peer = original_init_peer
    }

    ffi::NGX_OK as isize
}

#[no_mangle]
unsafe extern "C" fn http_upstream_init_custom_peer(
    r: *mut ffi::ngx_http_request_t,
    us: *mut ffi::ngx_http_upstream_srv_conf_t,
) -> ffi::ngx_int_t {
    let req = RequestRef::from_ptr_mut(r);
    let us = upstream::SrvConfRef::from_ptr(us);

    req.connection().log().http().debug("custom init peer");

    let hccf = us.srv_conf::<SrvConfig>(ModuleRef::from_ptr(
        &mut ngx_http_upstream_custom_module as *mut _,
    ));

    if let Some(hccf) = hccf {
        if let Some(f) = hccf.original_init_peer {
            if f(req.as_ptr(), us.as_ptr()) != ffi::NGX_OK as isize {
                req.connection()
                    .log()
                    .http()
                    .debug("failed calling init_peer");

                return ffi::NGX_ERROR as isize;
            }
        }
    } else {
        req.connection().log().http().emerg("no upstream srv_conf");

        return ffi::NGX_ERROR as isize;
    };

    let hcpd = req
        .pool()
        .allocate(CustomPeerData {
            conf: hccf.and_then(|r| NonNull::new(r as *const _ as *mut _)),
            upstream: req
                .upstream()
                .and_then(|r| NonNull::new(r as *const _ as *mut _)),
            client_connection: NonNull::new(req.connection() as *const _ as *mut _),
            original_get_peer: None,
            original_free_peer: None,
        })
        .unwrap() as *mut _ as *mut _;

    if let Some(us) = req.upstream_mut() {
        let peer = us.peer_mut();
        peer.data = hcpd;
        peer.get = Some(ngx_http_upstream_get_custom_peer);
        peer.free = Some(ngx_http_upstream_free_custom_peer);
    } else {
        req.connection().log().http().emerg("no upstream");

        return ffi::NGX_ERROR as isize;
    }

    ffi::NGX_OK as isize
}

unsafe extern "C" fn ngx_http_upstream_get_custom_peer(
    pc: *mut ffi::ngx_peer_connection_t,
    data: *mut c_void,
) -> ffi::ngx_int_t {
    let pc = PeerConnRef::from_ptr(pc);
    0
}

unsafe extern "C" fn ngx_http_upstream_free_custom_peer(
    pc: *mut ffi::ngx_peer_connection_t,
    data: *mut c_void,
    state: ffi::ngx_uint_t,
) {
    let pc = PeerConnRef::from_ptr(pc);
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct CustomPeerData {
    conf: Option<NonNull<SrvConfig>>,
    upstream: Option<NonNull<upstream::UpstreamRef>>,
    client_connection: Option<NonNull<ConnRef>>,
    original_get_peer: ffi::ngx_event_get_peer_pt,
    original_free_peer: ffi::ngx_event_free_peer_pt,
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
