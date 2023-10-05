#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use std::ptr::NonNull;

use anyhow::anyhow;
use foreign_types::ForeignTypeRef;
use merge::Merge as AutoMerge;

use ngx_mod::{
    core::Setter,
    http,
    rt::{
        core::{
            conf::{self, Unset},
            CmdRef, ConfRef, ConnRef,
        },
        event::{FreePeerFn, GetPeerFn, PeerConnRef},
        ffi::{self, ngx_command_t},
        http::{
            upstream::{self, InitPeerFn},
            RequestRef,
        },
        native_handler, ngx_str,
    },
    Conf, Merge, Module, ModuleMetadata as _,
};

#[derive(Module)]
#[module(name = http_upstream_custom, type = http)]
struct Custom;

impl Module for Custom {}

impl http::Module for Custom {
    type Error = ();
    type MainConf = ();
    type SrvConf = SrvConfig;
    type LocConf = ();

    fn create_srv_conf(cf: &ConfRef) -> Option<&mut Self::SrvConf> {
        if let Some(p) = cf.pool().allocate_default::<SrvConfig>() {
            p.max = conf::unset();

            Some(p)
        } else {
            cf.emerg("could not allocate memory for config, out of memory");

            None
        }
    }
}

#[derive(Clone, Debug, AutoMerge, Conf)]
struct SrvConfig {
    #[merge(strategy = merge::num::overwrite_zero)]
    max: u32,
    original_init_upstream: Option<upstream::InitFn>,
    original_init_peer: Option<upstream::InitPeerFn>,
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

        let uscf = cf
            .as_http_context()
            .expect("ctx")
            .srv_conf_for::<upstream::SrvConfRef>(Custom::module())
            .expect("srvConf");

        conf.original_init_upstream = uscf.peer().init_upstream().or(Some(upstream::InitFn(
            ffi::ngx_http_upstream_init_round_robin,
        )));

        uscf.peer_mut().init_upstream = Some(ngx_http_upstream_init_custom);

        Ok(())
    }
}

#[native_handler(name = ngx_http_upstream_init_custom, log_err = cf.emerg)]
fn init_custom(cf: &ConfRef, us: &mut upstream::SrvConfRef) -> anyhow::Result<()> {
    cf.log().http().debug("custom init upstream");

    let original_init_upstream = {
        let hccf = us
            .srv_conf_mut::<SrvConfig>(Custom::module())
            .ok_or_else(|| anyhow!("no upstream srv_conf"))?;

        hccf.max.get_or_set(100);
        hccf.original_init_upstream
    };

    if let Some(f) = original_init_upstream {
        f.call(cf, us)
            .map_err(|_| anyhow!("failed calling init_upstream"))?;
    }

    let original_init_peer = us.peer_mut().init.replace(http_upstream_init_custom_peer);

    if let Some(hccf) = us.srv_conf_mut::<SrvConfig>(Custom::module()) {
        hccf.original_init_peer = original_init_peer.map(InitPeerFn)
    }

    Ok(())
}

#[native_handler(name = http_upstream_init_custom_peer, log_err = req.connection().log().http().emerg)]
fn init_custom_peer(req: &mut RequestRef, us: &upstream::SrvConfRef) -> anyhow::Result<()> {
    req.connection().log().http().debug("custom init peer");

    let hccf = us.srv_conf::<SrvConfig>(Custom::module());

    if let Some(f) = hccf
        .ok_or_else(|| anyhow!("no upstream srv_conf"))?
        .original_init_peer
    {
        f.call(req, us)
            .map_err(|_| anyhow!("failed calling init_peer"))?;
    }

    let hcpd = req
        .pool()
        .allocate(UpstreamPeerData {
            conf: hccf,
            upstream: req.upstream(),
            client_connection: Some(req.connection()),
            original_get_peer: req.upstream().and_then(|us| us.peer().get()),
            original_free_peer: req.upstream().and_then(|us| us.peer().free()),
            data: req.upstream().and_then(|us| us.peer().data()),
        })
        .and_then(|r| NonNull::new(r as *mut _).map(|p| p.cast()))
        .ok_or_else(|| anyhow!("out of memory"))?;

    let us = req.upstream_mut().ok_or_else(|| anyhow!("no upstream"))?;
    let peer = us.peer_mut();
    peer.data = hcpd.as_ptr();
    peer.get = Some(ngx_http_upstream_get_custom_peer);
    peer.free = Some(ngx_http_upstream_free_custom_peer);

    Ok(())
}

#[native_handler(name = ngx_http_upstream_get_custom_peer, log_err = conn.log().http().emerg)]
fn get_custom_peer(conn: &PeerConnRef, data: &UpstreamPeerData) -> anyhow::Result<()> {
    conn.log().http().debug(format!(
        "custom get peer, try: {}, conn: {:p}",
        conn.tries,
        conn.as_ptr()
    ));

    if let Some(f) = data.original_get_peer {
        f.call(conn, Some(data))
            .map_err(|_| anyhow!("failed calling get_peer"))?;
    }

    /* in this section you can set the upstream server connection */

    Ok(())
}

#[native_handler(name = ngx_http_upstream_free_custom_peer)]
fn free_custom_peer(pc: &PeerConnRef, data: &UpstreamPeerData, state: usize) {
    pc.log().http().debug("custom free peer");

    if let Some(f) = data.original_free_peer {
        f.call(pc, Some(data), state);
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct UpstreamPeerData<'a> {
    conf: Option<&'a SrvConfig>,
    upstream: Option<&'a upstream::UpstreamRef>,
    client_connection: Option<&'a ConnRef>,
    original_get_peer: Option<GetPeerFn>,
    original_free_peer: Option<FreePeerFn>,
    data: Option<NonNull<()>>,
}

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
