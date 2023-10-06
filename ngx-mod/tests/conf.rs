#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use ngx_mod::Conf;

#[derive(Clone, Debug, Conf)]
#[conf(http::upstream)]
pub struct MainConf {
    #[directive(http::server, args(0, 1))]
    pub max: isize,
}
