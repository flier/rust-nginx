#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use std::fmt::Display;
use std::marker::PhantomData;
use std::process;
use std::ptr;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context as _};
use foreign_types::ForeignTypeRef;
use opentelemetry::{
    global,
    propagation::{Extractor, Injector},
    runtime,
    sdk::{
        propagation::TraceContextPropagator,
        trace::{config as trace_config, BatchConfig},
        Resource,
    },
    trace::{
        Span, SpanBuilder, SpanContext, SpanId, SpanKind, TraceContextExt, TraceFlags, TraceId,
        TraceState, Tracer,
    },
    Context, Key, KeyValue, OrderMap, Value,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions as semcov;
use static_str_ops::staticize;

use ngx_mod::{
    http::{self, Module as _},
    rt::{
        core::{time::MSec, ArrayRef, CmdRef, Code, ConfRef, CycleRef, Str, Unset},
        debug, error,
        http::{
            core::{self, Phases},
            script::{self, ComplexValueRef},
            Headers, ValueRef,
        },
        info, native_setter, ngx_var, Error,
    },
    Conf, Merge, Module,
};
use ngx_rt::{http::RequestRef, native_handler};

#[derive(Module)]
#[module(name = http_otel, type = http)]
struct Otel<'a>(PhantomData<&'a u8>);

impl<'a> Module for Otel<'a> {
    fn init_process(cycle: &CycleRef) -> Result<(), Code> {
        info!(cycle, "otel: init process {}", process::id());

        if let Some(mcf) = Self::main_conf(cycle) {
            if !mcf.exporter.endpoint.is_empty() {
                mcf.init_tracing().map_err(|err| {
                    error!(cycle, "otel: failed to initialize opentelemetry: {}", err);

                    Code::ERROR
                })?;
            }
        }

        Ok(())
    }

    fn exit_process(cycle: &CycleRef) {
        info!(cycle, "otel: exit process {}", process::id());

        global::shutdown_tracer_provider();
    }
}

impl<'a> http::Module for Otel<'a> {
    type Error = ();
    type MainConf = MainConf;
    type SrvConf = ();
    type LocConf = LocConf<'a>;

    fn preconfiguration(cf: &ConfRef) -> Result<(), Code> {
        info!(cf, "otel: preconfiguration");

        cf.add_variables([
            ngx_var!("otel_trace_id", get = current_trace_id),
            ngx_var!("otel_span_id", get = current_span_id),
            ngx_var!("otel_parent_id", get = parent_span_id),
            ngx_var!("otel_parent_sampled", get = parent_sampled_var),
        ])
        .map_err(|_| Code::ERROR)
    }

    fn postconfiguration(cf: &ConfRef) -> Result<(), Code> {
        info!(cf, "otel: postconfiguration");

        let cmcf = cf
            .as_http_context()
            .and_then(core::main_conf_mut)
            .ok_or(Code::ERROR)?;

        cmcf.push_handler(Phases::Rewrite, otel_request_start);
        cmcf.push_handler(Phases::Log, otel_request_end);

        Ok(())
    }

    fn init_main_conf(cf: &ConfRef, conf: &mut Self::MainConf) -> Result<(), Self::Error> {
        info!(cf, "otel: init main conf");

        conf.exporter.interval.or_insert(5000.into());
        conf.exporter.batch_size.or_insert(512);
        conf.exporter.batch_count.or_insert(4);
        conf.service_name
            .or_insert_with(|| Str::from("unknown_service:nginx"));

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct TraceContext {
    trace_id: TraceId,
    span_id: SpanId,
    sampled: bool,
    trace_state: TraceState,
}

impl Default for TraceContext {
    fn default() -> Self {
        (&SpanContext::empty_context()).into()
    }
}

impl<'a> From<&'a SpanContext> for TraceContext {
    fn from(ctx: &'a SpanContext) -> Self {
        TraceContext {
            trace_id: ctx.trace_id(),
            span_id: ctx.span_id(),
            sampled: ctx.is_sampled(),
            trace_state: ctx.trace_state().clone(),
        }
    }
}

impl<'a> From<&'a TraceContext> for SpanContext {
    fn from(ctx: &'a TraceContext) -> Self {
        SpanContext::new(
            ctx.trace_id,
            ctx.span_id,
            if ctx.sampled {
                TraceFlags::SAMPLED
            } else {
                TraceFlags::default()
            },
            ctx.trace_id != TraceId::INVALID,
            ctx.trace_state.clone(),
        )
    }
}

impl TraceContext {
    pub fn generate(sampled: bool, parent: SpanContext) -> TraceContext {
        let gen = &trace_config().id_generator;

        TraceContext {
            trace_id: if parent.is_valid() {
                parent.trace_id()
            } else {
                gen.new_trace_id()
            },
            span_id: gen.new_span_id(),
            sampled,
            trace_state: parent.trace_state().clone(),
        }
    }

    pub fn extract(req: &RequestRef) -> TraceContext {
        let ctx = global::get_text_map_propagator(|propagator| {
            let extractor = HeaderExtractor(req.headers());

            propagator.extract(&extractor)
        });

        ctx.span().span_context().into()
    }

    pub fn inject(&self, req: &RequestRef) {
        Context::map_current(|ctx| {
            let ctx = ctx.with_remote_span_context(self.into());

            global::get_text_map_propagator(|propagator| {
                let mut injector = HeaderInjector(req.headers());

                propagator.inject_context(&ctx, &mut injector)
            });
        })
    }
}

struct HeaderExtractor<'a>(Headers<'a>);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|h| h.value().to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().flat_map(|h| h.lowcase_key()).collect()
    }
}

struct HeaderInjector<'a>(Headers<'a>);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key, &value);
    }
}

#[derive(Clone, Debug, Default)]
struct OtelContext {
    parent: TraceContext,
    current: TraceContext,
}

impl OtelContext {
    pub fn create(req: &RequestRef) -> Option<&mut OtelContext> {
        debug!(req, "otel: creating context");

        if let Some(p) = unsafe {
            req.pool()
                .add_cleanup::<OtelContext>(Some(cleanup_otel_ctx), None)
                .ok()?
        } {
            let ctx = p.write(OtelContext::default());

            debug!(req, "otel: set context to @ {:p}", ctx as *mut _);

            Otel::set_module_ctx(req, ctx);

            Some(ctx)
        } else {
            None
        }
    }

    pub fn get(req: &RequestRef) -> Option<&mut OtelContext> {
        Otel::module_ctx_mut(req).or_else(|| {
            if req.internal() || req.filter_finalize() {
                req.pool()
                    .cleanups()
                    .find(|c| c.raw_handler() == Some(cleanup_otel_ctx))
                    .and_then(|p| {
                        if let Some(ctx) = p.data() {
                            debug!(req, "otel: set context to @ {:p}", ctx as *const _);

                            Otel::set_module_ctx(req, ctx);

                            Some(ctx)
                        } else {
                            None
                        }
                    })
            } else {
                None
            }
        })
    }

    pub fn ensure(req: &RequestRef) -> Option<&mut OtelContext> {
        let ctx = OtelContext::get(req).or(OtelContext::create(req))?;

        let lcf = Otel::loc_conf(req)?;

        if lcf.trace_ctx().contains(propagation::Type::EXTRACT) {
            ctx.parent = TraceContext::extract(req);
        }

        ctx.current = TraceContext::generate(false, (&ctx.parent).into());

        return Some(ctx);
    }
}

#[native_handler]
fn cleanup_otel_ctx(data: &mut OtelContext) {
    unsafe { ptr::drop_in_place(data as *mut _) };
}

fn id_var<F, T>(req: &RequestRef, val: &mut ValueRef, f: F) -> Result<(), Code>
where
    F: FnOnce(&OtelContext) -> Option<&T>,
    T: Display,
{
    let ctx = OtelContext::ensure(req).ok_or(Code::ERROR)?;

    if let Some(id) = f(ctx) {
        let id = req
            .pool()
            .strdup(id.to_string().to_lowercase())
            .ok_or(Code::ERROR)?;

        val.set_value(id);
    } else {
        val.set_not_found(true);
    }

    Ok(())
}

#[native_handler]
fn current_trace_id(req: &RequestRef, val: &mut ValueRef, _data: usize) -> Result<(), Code> {
    id_var(req, val, |ctx| Some(&ctx.current.trace_id))
}

#[native_handler]
fn current_span_id(req: &RequestRef, val: &mut ValueRef, _data: usize) -> Result<(), Code> {
    id_var(req, val, |ctx| Some(&ctx.current.span_id))
}

#[native_handler]
fn parent_span_id(req: &RequestRef, val: &mut ValueRef, _data: usize) -> Result<(), Code> {
    id_var(req, val, |ctx| Some(&ctx.parent.span_id))
}

#[native_handler]
fn parent_sampled_var(req: &RequestRef, val: &mut ValueRef, _data: usize) -> Result<(), Code> {
    let ctx = OtelContext::ensure(req).ok_or(Code::ERROR)?;

    val.set_value(if ctx.parent.sampled { "on" } else { "off" });

    Ok(())
}

#[repr(C)]
#[derive(Clone, Conf)]
#[conf(http::main, default = unset)]
struct MainConf {
    #[directive(name = "otel_exporter", args(0), block, set = set_exporter)]
    exporter: Exporter,
    #[directive(name = "otel_service_name", args(1))]
    service_name: Str,
}

impl MainConf {
    pub fn init_tracing(&self) -> anyhow::Result<()> {
        global::set_text_map_propagator(TraceContextPropagator::new());

        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(self.exporter.endpoint.to_str()?.to_string()),
            )
            .with_trace_config(trace_config().with_resource(Resource::new(vec![
                semcov::resource::SERVICE_NAME.string(self.service_name.to_str()?.to_string()),
            ])))
            .with_batch_config(
                BatchConfig::default()
                    .with_max_export_batch_size(self.exporter.batch_size)
                    .with_max_concurrent_exports(self.exporter.batch_count)
                    .with_scheduled_delay(self.exporter.interval.into()),
            )
            .install_batch(runtime::Tokio)?;

        Ok(())
    }
}

#[native_setter(log = cf)]
fn set_exporter(cf: &ConfRef, _cmd: &CmdRef, conf: &mut MainConf) -> anyhow::Result<()> {
    if !conf.exporter.endpoint.is_empty() {
        bail!("exporter is duplicate");
    }

    cf.parse_block(&mut conf.exporter)
        .context("parse exporter")?;

    Ok(())
}

#[repr(C)]
#[derive(Clone, Conf)]
#[conf(default = unset)]
struct Exporter {
    #[directive(args(1), set = str)]
    endpoint: Str,
    #[directive(args(1), set = msec)]
    interval: MSec,
    #[directive(args(1), set = size)]
    batch_size: usize,
    #[directive(args(1), set = size)]
    batch_count: usize,
}

#[repr(C)]
#[derive(Clone, Conf)]
#[conf(http::main | http::server | http::location, default = unset)]
struct LocConf<'a> {
    #[directive(name = "otel_trace", args(1), set = complex_value)]
    trace: Option<&'a ComplexValueRef>,
    #[directive(name = "otel_trace_context", args(1), set = enum_values, values = propagation::TYPES)]
    trace_ctx: usize,
    #[directive(name = "otel_span_name", args(1), set = complex_value)]
    span_name: Option<&'a ComplexValueRef>,
    #[directive(name = "otel_span_attr", args(2), set = add_span_attr)]
    span_attrs: <ArrayRef<SpanAttr> as ForeignTypeRef>::CType,
}

impl LocConf<'_> {
    pub fn trace_ctx(&self) -> propagation::Type {
        propagation::Type::from_bits_truncate(self.trace_ctx)
    }

    pub fn span_attrs(&self) -> &ArrayRef<SpanAttr> {
        unsafe { ArrayRef::from_ptr(&self.span_attrs as *const _ as *mut _) }
    }

    pub fn span_attrs_mut(&mut self) -> &mut ArrayRef<SpanAttr> {
        unsafe { ArrayRef::from_ptr_mut(&mut self.span_attrs as *mut _) }
    }
}

impl Merge for LocConf<'_> {
    type Error = ();

    fn merge(&mut self, prev: &Self) -> Result<(), ()> {
        self.trace.or_insert(prev.trace);
        self.trace_ctx.or_insert(prev.trace_ctx);
        self.span_name.or_insert(prev.span_name);
        self.span_attrs.or_insert(prev.span_attrs);

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
struct SpanAttr {
    key: Str,
    value: <ComplexValueRef as ForeignTypeRef>::CType,
}

mod propagation {
    use ngx_mod::rt::ngx_enum_values;

    bitflags::bitflags! {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct Type: usize {
            const EXTRACT = 0x0001;
            const INJECT = 0x0002;
        }
    }

    ngx_enum_values! {
        pub enum TYPES {
            "ignore" => Type::empty().bits(),
            "extract" => Type::EXTRACT.bits(),
            "inject" => Type::INJECT.bits(),
            "propagate" => Type::EXTRACT.bits() | Type::INJECT.bits()
        }
    }
}

#[native_setter(log = cf)]
fn add_span_attr(cf: &ConfRef, _cmd: &CmdRef, conf: &mut LocConf) -> anyhow::Result<()> {
    let span_attrs = conf.span_attrs_mut();
    if span_attrs.is_null() {
        span_attrs.init(cf.pool(), 4).ok_or(Error::OutOfMemory)?;
    }

    let (key, value) = cf
        .args()
        .get(1)
        .zip(cf.args().get(2))
        .ok_or_else(|| anyhow!("missing arguments"))?;

    let v = script::ComplexValueCompiler::new(cf)
        .compile(value)
        .map_err(|_| anyhow!("failed to compile complex value"))?;

    span_attrs
        .push(SpanAttr {
            key: key.clone(),
            value: v,
        })
        .ok_or(Error::OutOfMemory)?;

    Ok(())
}

#[native_handler(name = otel_request_start)]
fn request_start(req: &RequestRef) -> Result<(), Code> {
    if req.internal() {
        return Err(Code::DECLINED);
    }

    let lcf = Otel::loc_conf(req).ok_or(Code::ERROR)?;

    let sampled = if let Some(v) = lcf.trace {
        let trace = v.evaluate(req).map_err(|_| Code::ERROR)?;

        trace == "on" || trace == "1"
    } else {
        false
    };

    if !lcf.trace_ctx.is_unset() && lcf.trace_ctx().is_empty() && !sampled {
        return Err(Code::DECLINED);
    }

    let ctx = OtelContext::ensure(req).ok_or(Code::ERROR)?;

    ctx.current.sampled = sampled;

    if lcf.trace_ctx().contains(propagation::Type::INJECT) {
        ctx.current.inject(req);
    }

    Ok(())
}

#[native_handler(name = otel_request_end)]
fn request_end(req: &RequestRef) -> Result<(), Code> {
    let ctx = OtelContext::get(req).ok_or(Code::DECLINED)?;

    if !ctx.current.sampled {
        return Err(Code::DECLINED);
    }

    let tracer = global::tracer("nginx/otel");
    let mut span = {
        tracer.build(SpanBuilder {
            trace_id: Some(ctx.current.trace_id),
            span_id: Some(ctx.current.span_id),
            span_kind: Some(SpanKind::Server),
            start_time: Some(
                SystemTime::UNIX_EPOCH
                    + Duration::new(req.start_sec() as u64, req.start_msec() as u32 * 1000_000),
            ),
            name: staticize(
                get_span_name(req)
                    .map_err(|err| {
                        error!(req, "failed to get span name: {}", err);

                        Code::ERROR
                    })?
                    .as_str(),
            )
            .into(),
            attributes: get_span_attrs(req).map(Some).map_err(|err| {
                error!(req, "failed to get span attrs: {}", err);

                Code::ERROR
            })?,
            ..Default::default()
        })
    };

    span.end();

    Ok(())
}

fn get_span_name(req: &RequestRef) -> anyhow::Result<String> {
    if let Some(v) = Otel::loc_conf(req).and_then(|lc| lc.span_name) {
        Ok(v.evaluate(req)?.to_str()?.to_string())
    } else {
        let lc = core::loc_conf(req).ok_or_else(|| anyhow!("missing `loc_conf`"))?;

        Ok(lc.name().to_str()?.to_string())
    }
}

fn get_span_attrs(req: &RequestRef) -> anyhow::Result<OrderMap<Key, Value>> {
    let mut attrs = vec![
        semcov::trace::HTTP_REQUEST_METHOD.string(req.method_name().to_str()?.to_string()),
        semcov::trace::URL_PATH.string(req.uri().to_str()?.to_string()),
    ];

    if let Some(args) = req.args() {
        attrs.push(semcov::trace::URL_QUERY.string(args.to_str()?.to_string()));
    }

    if let Some(name) = core::loc_conf(req).map(|lc| lc.name()) {
        attrs.push(semcov::trace::HTTP_ROUTE.string(name.to_str()?.to_string()));
    }

    let s = req.http_protocol().to_str()?;
    if s.len() > 5 {
        let (_, v) = s.split_at("HTTP/".len());
        attrs.push(semcov::trace::NETWORK_PROTOCOL_VERSION.string(v.to_string()));
    }

    if let Some(ua) = req.user_agent().map(|h| h.value()) {
        attrs.push(semcov::trace::USER_AGENT_ORIGINAL.string(ua.to_str()?.to_string()));
    }

    if req.content_length_n() > 0 {
        attrs.push(semcov::trace::HTTP_REQUEST_BODY_SIZE.i64(req.content_length_n()));
    }

    if let Some(sent) = req
        .connection()
        .sent()
        .checked_sub(req.header_size() as i64)
    {
        attrs.push(semcov::trace::HTTP_RESPONSE_BODY_SIZE.i64(sent));
    }

    if let Some(status) = if req.err_status() != 0 {
        Some(req.err_status())
    } else if req.headers_out().status() != 0 {
        Some(req.headers_out().status())
    } else {
        None
    } {
        attrs.push(semcov::trace::HTTP_RESPONSE_STATUS_CODE.i64(status as i64));
    }

    if let Some(name) = core::srv_conf(req)
        .and_then(|sc| sc.server_name())
        .or(req.server())
    {
        attrs.push(semcov::trace::SERVER_ADDRESS.string(name.to_str()?.to_string()));
    }

    if let Some(addr) = req.connection().local() {
        attrs.push(semcov::trace::SERVER_ADDRESS.string(addr.ip().to_string()));
        attrs.push(semcov::trace::SERVER_PORT.i64(addr.port() as i64));
    }

    if let Some(addr) = req.connection().remote() {
        attrs.push(semcov::trace::CLIENT_ADDRESS.string(addr.ip().to_string()));
        attrs.push(semcov::trace::CLIENT_PORT.i64(addr.port() as i64));
    }

    if let Some(lc) = Otel::loc_conf(req) {
        for attr in lc.span_attrs().iter() {
            let value = unsafe { ComplexValueRef::from_ptr(&attr.value as *const _ as *mut _) };
            let value = value.evaluate(req)?;

            attrs.push(KeyValue::new(
                attr.key.to_str()?.to_string(),
                value.to_str()?.to_string(),
            ))
        }
    }

    Ok(attrs.into_iter().collect())
}
