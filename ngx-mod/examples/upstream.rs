#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use std::ptr::NonNull;

use anyhow::{anyhow, bail, Context};
use merge::Merge as AutoMerge;

use ngx_mod::{
    http,
    rt::{
        core::{
            conf::{self, Unset},
            CmdRef, ConfRef, ConnRef,
        },
        event::{FreePeerFn, GetPeerFn, PeerConnRef},
        ffi,
        http::{upstream, RequestRef},
        native_handler, native_setter,
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
}

#[derive(Clone, Debug, AutoMerge, Conf)]
#[conf(http::upstream)]
struct SrvConfig {
    #[directive(name = "custom", args(0, 1), set = ngx_http_upstream_custom)]
    #[merge(strategy = overwrite_unset)]
    max: usize,
    original_init_upstream: Option<upstream::InitFn>,
    original_init_peer: Option<upstream::InitPeerFn>,
}
fn overwrite_unset<T>(left: &mut T, right: T)
where
    T: Sized + PartialEq + Unset,
{
    if left.is_unset() {
        *left = right;
    }
}

impl Default for SrvConfig {
    fn default() -> Self {
        SrvConfig {
            max: conf::unset(),
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

#[native_setter(name = ngx_http_upstream_custom, log_err = cf.emerg)]
fn set_custom(cf: &ConfRef, _cmd: &CmdRef, conf: &mut SrvConfig) -> anyhow::Result<()> {
    cf.notice("CUSTOM init module");

    if let Some(s) = cf.args().get(1) {
        let n = s
            .to_str()?
            .parse()
            .context("expect `custom` value to be a number")?;
        if n > 0 {
            conf.max = n;
        } else {
            bail!("expect `custom` value to be greater than 0");
        }
    }

    let uscf = cf
        .as_http_context()
        .and_then(|ctx| ctx.srv_conf_for::<upstream::SrvConfRef>(upstream::module()))
        .ok_or_else(|| anyhow!("`srv_conf` not found"))?;

    conf.original_init_upstream = uscf.peer().init_upstream().or(Some(upstream::InitFn(
        ffi::ngx_http_upstream_init_round_robin,
    )));

    uscf.peer_mut().init_upstream = Some(ngx_http_upstream_init_custom);

    Ok(())
}

#[native_handler(name = ngx_http_upstream_init_custom, log_err = cf.emerg)]
fn init_custom(cf: &ConfRef, us: &mut upstream::SrvConfRef) -> anyhow::Result<()> {
    cf.notice("CUSTOM init upstream");

    let original_init_upstream = {
        let hccf = us
            .srv_conf_mut::<SrvConfig>(Custom::module())
            .ok_or_else(|| anyhow!("no upstream `srv_conf`"))?;

        hccf.max.get_or_set(100);
        hccf.original_init_upstream
    };

    if let Some(f) = original_init_upstream {
        f.call(cf, us)
            .map_err(|_| anyhow!("failed calling `init_upstream`"))?;
    }

    let original_init_peer = us.peer_mut().init.replace(http_upstream_init_custom_peer);

    if let Some(hccf) = us.srv_conf_mut::<SrvConfig>(Custom::module()) {
        hccf.original_init_peer = original_init_peer.map(upstream::InitPeerFn)
    }

    Ok(())
}

#[native_handler(name = http_upstream_init_custom_peer, log_err = req.connection().log().http().emerg)]
fn init_custom_peer(req: &mut RequestRef, us: &upstream::SrvConfRef) -> anyhow::Result<()> {
    req.connection().log().http().debug("CUSTOM init peer");

    let hccf = us.srv_conf::<SrvConfig>(Custom::module());

    if let Some(f) = hccf
        .ok_or_else(|| anyhow!("no upstream srv_conf"))?
        .original_init_peer
    {
        f.call(req, us)
            .map_err(|_| anyhow!("failed calling `init_peer`"))?;
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
        "CUSTOM get peer, try: {}, conn: {:p}",
        conn.tries, conn
    ));

    if let Some(f) = data.original_get_peer {
        f.call(conn, data.data)
            .map_err(|_| anyhow!("failed calling get_peer"))?;
    }

    /* in this section you can set the upstream server connection */

    Ok(())
}

#[native_handler(name = ngx_http_upstream_free_custom_peer)]
fn free_custom_peer(conn: &PeerConnRef, data: &UpstreamPeerData, state: usize) {
    conn.log()
        .http()
        .debug(format!("CUSTOM free peer, conn: {:p}", conn));

    if let Some(f) = data.original_free_peer {
        f.call(conn, data.data, state);
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
    data: Option<&'a ()>,
}
