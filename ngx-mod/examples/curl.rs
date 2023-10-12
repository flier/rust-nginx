#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use http::StatusCode;
use merge::Merge as AutoMerge;

use ngx_mod::{
    http::Module as HttpModule,
    rt::{
        core::{CmdRef, Code, ConfRef},
        http::core::{self, Phases},
        http_debug, native_setter, notice,
    },
    Conf, Merge, Module,
};
use ngx_rt::{http::RequestRef, native_handler};

#[derive(Module)]
#[module(name = ngx_http_curl, type = http)]
struct Curl;

impl Module for Curl {}

impl HttpModule for Curl {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
    type LocConf = LocConfig;

    fn postconfiguration(cf: &ConfRef) -> Result<(), Code> {
        notice!(cf, "CURL init module");

        let cmcf = cf
            .as_http_context()
            .map(core::main_conf_mut)
            .ok_or(Code::ERROR)?;

        cmcf.phases_mut(Phases::Access)
            .handlers_mut()
            .push(Some(ngx_http_curl_access_handler));

        Ok(())
    }
}

#[derive(Clone, Debug, Default, AutoMerge, Conf)]
#[conf(http::location)]
struct LocConfig {
    #[directive(name = "curl", args(1), set = ngx_http_curl_commands_set_enable)]
    #[merge(strategy = merge::bool::overwrite_false)]
    enable: bool,
}

impl Merge for LocConfig {
    type Error = ();

    fn merge(&mut self, prev: &LocConfig) -> Result<(), ()> {
        merge::Merge::merge(self, prev.clone());

        Ok(())
    }
}

#[native_setter(name = ngx_http_curl_commands_set_enable, log = cf)]
fn set_enable(cf: &ConfRef, _cmd: &CmdRef, conf: &mut LocConfig) -> anyhow::Result<()> {
    notice!(cf, "CURL set enable");

    conf.enable = if let Some(s) = cf.args().get(1) {
        s.to_str()?.eq_ignore_ascii_case("on")
    } else {
        false
    };

    Ok(())
}

#[native_handler(name = ngx_http_curl_access_handler)]
fn curl_access(req: &RequestRef) -> Result<StatusCode, Code> {
    let lc = Curl::loc_conf(req);

    http_debug!(req, "CURL enabled: {}", lc.enable);

    if lc.enable
        && req
            .user_agent()
            .and_then(|h| h.value())
            .map_or(false, |s| s.as_bytes().starts_with(b"curl"))
    {
        Ok(StatusCode::FORBIDDEN)
    } else {
        Err(Code::DECLINED)
    }
}
