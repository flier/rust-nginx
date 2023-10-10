#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use http::{HeaderMap, StatusCode};
use merge::Merge as AutoMerge;

use ngx_mod::{
    http::Module as HttpModule,
    rt::{
        core::{CmdRef, Code, ConfRef},
        debug,
        http::core::{self, Phases},
        native_setter, notice,
    },
    Conf, Merge, Module,
};
use ngx_rt::{http::RequestRef, native_handler};

#[derive(Module)]
#[module(name = ngx_http_awssigv4, type = http)]
struct AwsSig;

impl Module for AwsSig {}

impl HttpModule for AwsSig {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
    type LocConf = Config;

    fn postconfiguration(cf: &ConfRef) -> Result<(), Code> {
        notice!(cf, "AwsSig init module");

        let cmcf = cf
            .as_http_context()
            .map(core::main_conf)
            .ok_or(Code::ERROR)?;

        cmcf.phases_mut(Phases::Precontent)
            .handlers_mut()
            .push(Some(awssigv4_header_handler));

        Ok(())
    }
}

#[derive(Clone, Debug, Default, AutoMerge, Conf)]
#[conf(http::server, http::location)]
struct Config {
    #[directive(name = "awssigv4", args(1), set = ngx_http_awssigv4_commands_set_enable)]
    #[merge(strategy = merge::bool::overwrite_false)]
    enable: bool,
    #[directive(name = "awssigv4_access_key", args(1), set = ngx_http_awssigv4_commands_set_access_key)]
    access_key: Option<String>,
    #[directive(name = "awssigv4_secret_key", args(1), set = ngx_http_awssigv4_commands_set_secret_key)]
    secret_key: Option<String>,
    #[directive(name = "awssigv4_s3_bucket", args(1), set = ngx_http_awssigv4_commands_set_s3_bucket)]
    s3_bucket: Option<String>,
    #[directive(name = "awssigv4_s3_endpoint", args(1), set = ngx_http_awssigv4_commands_set_s3_endpoint)]
    s3_endpoint: Option<String>,
}

impl Merge for Config {
    type Error = ();

    fn merge(&mut self, prev: &Config) -> Result<(), ()> {
        merge::Merge::merge(self, prev.clone());

        Ok(())
    }
}

#[native_setter(name = ngx_http_awssigv4_commands_set_enable, log_err = cf.emerg)]
fn set_enable(cf: &ConfRef, _cmd: &CmdRef, conf: &mut Config) -> anyhow::Result<()> {
    conf.enable = if let Some(s) = cf.args().get(1) {
        s.to_str()?.eq_ignore_ascii_case("on")
    } else {
        false
    };

    notice!(cf, "AwsSig set enable: {}", conf.enable);

    Ok(())
}

#[native_setter(name = ngx_http_awssigv4_commands_set_access_key, log_err = cf.emerg)]
fn set_access_key(cf: &ConfRef, _cmd: &CmdRef, conf: &mut Config) -> anyhow::Result<()> {
    conf.access_key = cf
        .args()
        .get(1)
        .map(|s| s.to_str())
        .transpose()?
        .map(|s| s.to_string());

    notice!(cf, "AwsSig set access key: {:?}", conf.access_key);

    Ok(())
}

#[native_setter(name = ngx_http_awssigv4_commands_set_secret_key, log_err = cf.emerg)]
fn set_secret_key(cf: &ConfRef, _cmd: &CmdRef, conf: &mut Config) -> anyhow::Result<()> {
    conf.secret_key = cf
        .args()
        .get(1)
        .map(|s| s.to_str())
        .transpose()?
        .map(|s| s.to_string());

    notice!(cf, "AwsSig set secret key: {:?}", conf.secret_key);

    Ok(())
}

#[native_setter(name = ngx_http_awssigv4_commands_set_s3_bucket, log_err = cf.emerg)]
fn set_s3_bucket(cf: &ConfRef, _cmd: &CmdRef, conf: &mut Config) -> anyhow::Result<()> {
    conf.s3_bucket = cf
        .args()
        .get(1)
        .map(|s| s.to_str())
        .transpose()?
        .map(|s| s.to_string());

    notice!(cf, "AwsSig set S3 bucket: {:?}", conf.s3_bucket);

    Ok(())
}

#[native_setter(name = ngx_http_awssigv4_commands_set_s3_endpoint, log_err = cf.emerg)]
fn set_s3_endpoint(cf: &ConfRef, _cmd: &CmdRef, conf: &mut Config) -> anyhow::Result<()> {
    conf.s3_endpoint = cf
        .args()
        .get(1)
        .map(|s| s.to_str())
        .transpose()?
        .map(|s| s.to_string());

    notice!(cf, "AwsSig set S3 bucket: {:?}", conf.s3_endpoint);

    Ok(())
}

#[native_handler(name = awssigv4_header_handler, embedded)]
fn header_handler(req: &mut RequestRef) -> Result<Code, Code> {
    let conf = AwsSig::loc_conf(req);

    debug!(req.connection().log().http(), "AwsSig module: {:?}", conf);

    if !conf.enable {
        return Err(Code::DECLINED);
    }

    let method = req.as_method().ok_or(Code::DECLINED)?;

    if !matches!(method, http::Method::HEAD | http::Method::GET) {
        return Ok(StatusCode::FORBIDDEN.into());
    }

    let uri = format!(
        "https://{}.{}{}",
        conf.s3_bucket.as_ref().map(|s| s.as_str()).unwrap_or(""),
        conf.s3_endpoint.as_ref().map(|s| s.as_str()).unwrap_or(""),
        req.unparsed_uri().ok_or(Code::ERROR).and_then(|s| s
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| Code::DECLINED))?
    );

    let datetime = chrono::Utc::now();
    let datetime_now = datetime.format("%Y%m%dT%H%M%SZ").to_string();

    let signature = {
        // NOTE: aws_sign_v4::AwsSign::new() implementation requires a HeaderMap.
        // Iterate over requests headers_in and copy into HeaderMap
        // Copy only headers that will be used to sign the request
        let mut headers = HeaderMap::new();

        if let Some(s) = req.host().and_then(|h| h.value()) {
            if let Ok(s) = s.to_str() {
                if let Ok(val) = s.parse() {
                    headers.insert(http::header::HOST, val);
                }
            }
        }

        headers.insert("X-Amz-Date", datetime_now.parse().unwrap());

        aws_sign_v4::AwsSign::new(
            method.as_str(),
            &uri,
            &datetime,
            &headers,
            "us-east-1",
            conf.access_key.as_ref().map(|s| s.as_str()).unwrap_or(""),
            conf.secret_key.as_ref().map(|s| s.as_str()).unwrap_or(""),
            "s3",
            "",
        )
        .sign()
    };

    req.headers().add("authorization", signature.as_str());
    req.headers().add("X-Amz-Date", datetime_now.as_str());

    Ok(Code::OK)
}
