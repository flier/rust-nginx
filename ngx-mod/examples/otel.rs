#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use std::ptr;
use std::{fmt::Display, marker::PhantomData};

use anyhow::anyhow;
use foreign_types::ForeignTypeRef;
use opentelemetry::{
    propagation::{Extractor, Injector, TextMapPropagator},
    sdk::{propagation::TraceContextPropagator, trace::config as trace_config},
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};

use ngx_mod::{
    http::{self, Module as _},
    rt::{
        core::{conf, time::MSec, ArrayRef, CmdRef, Code, ConfRef, Str, Unset},
        debug,
        http::{
            core::{self, Phases},
            script::{self, ComplexValueRef},
            Headers, ValueRef,
        },
        native_setter, ngx_var, notice, Error,
    },
    Conf, Merge, Module,
};
use ngx_rt::{http::RequestRef, native_handler};

#[derive(Module)]
#[module(name = http_otel, type = http)]
struct Otel<'a>(PhantomData<&'a u8>);

impl<'a> Module for Otel<'a> {}

impl<'a> http::Module for Otel<'a> {
    type Error = ();
    type MainConf = MainConf;
    type SrvConf = ();
    type LocConf = LocConf<'a>;

    fn preconfiguration(cf: &ConfRef) -> Result<(), Code> {
        notice!(cf, "otel: preconf module");

        cf.add_variables([
            ngx_var!("otel_trace_id", get = current_trace_id),
            ngx_var!("otel_span_id", get = current_span_id),
            ngx_var!("otel_parent_id", get = parent_span_id),
            ngx_var!("otel_parent_sampled", get = parent_sampled_var),
        ])
        .map_err(|_| Code::ERROR)
    }

    fn postconfiguration(cf: &ConfRef) -> Result<(), Code> {
        notice!(cf, "otel: postconf module");

        let cmcf = cf
            .as_http_context()
            .map(core::main_conf_mut)
            .ok_or(Code::ERROR)?;

        cmcf.phases_mut(Phases::Rewrite)
            .handlers_mut()
            .push(Some(otel_request_start));

        cmcf.phases_mut(Phases::Log)
            .handlers_mut()
            .push(Some(otel_request_end));

        Ok(())
    }

    fn init_main_conf(_cf: &ConfRef, conf: &mut Self::MainConf) -> Result<(), Self::Error> {
        conf.interval.or_insert(5000);
        conf.batch_size.or_insert(512);
        conf.batch_count.or_insert(4);
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
        let propagator = TraceContextPropagator::new();
        let extractor = HttpHeaders(req.headers());
        let ctx = propagator.extract(&extractor);

        ctx.span().span_context().into()
    }

    pub fn inject(&self, req: &RequestRef) {
        let propagator: TraceContextPropagator = TraceContextPropagator::new();
        let mut injector = HttpHeaders(req.headers());

        Context::map_current(|ctx| {
            let ctx = ctx.with_remote_span_context(self.into());

            propagator.inject_context(&ctx, &mut injector)
        })
    }
}

struct HttpHeaders<'a>(Headers<'a>);

impl<'a> Extractor for HttpHeaders<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0
            .find(key)
            .and_then(|h| h.value())
            .and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().flat_map(|h| h.lowcase_key()).collect()
    }
}

impl<'a> Injector for HttpHeaders<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.set(key, &value);
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

        let lcf = Otel::loc_conf(req);

        if lcf.trace_ctx.contains(propagation::Type::EXTRACT) {
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
#[conf(http::main)]
struct MainConf {
    #[directive(args(1))]
    endpoint: Str,
    #[directive(args(1))]
    interval: MSec,
    #[directive(args(1))]
    batch_size: usize,
    #[directive(args(1))]
    batch_count: usize,
    #[directive(args(1))]
    service_name: Str,
}

impl Default for MainConf {
    fn default() -> Self {
        MainConf {
            endpoint: conf::unset(),
            interval: conf::unset(),
            batch_size: conf::unset(),
            batch_count: conf::unset(),
            service_name: conf::unset(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Conf)]
#[conf(http::main, http::server, http::location)]
struct LocConf<'a> {
    #[directive(args(1), set = complex_value)]
    trace: Option<&'a ComplexValueRef>,
    #[directive(args(1), set = enum_values, values = propagation::TYPES)]
    trace_ctx: propagation::Type,
    #[directive(args(1), set = complex_value)]
    span_name: Option<&'a ComplexValueRef>,
    #[directive(args(2), set = add_span_attr)]
    span_attrs: <ArrayRef<SpanAttr> as ForeignTypeRef>::CType,
}

impl Default for LocConf<'_> {
    fn default() -> Self {
        LocConf {
            trace: conf::unset(),
            trace_ctx: conf::unset(),
            span_name: conf::unset(),
            span_attrs: conf::unset(),
        }
    }
}

impl LocConf<'_> {
    pub fn span_attrs(&mut self) -> &mut ArrayRef<SpanAttr> {
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
    name: Str,
    value: <ComplexValueRef as ForeignTypeRef>::CType,
}

mod propagation {
    use ngx_mod::rt::{core::Unset, ngx_enum_values};

    bitflags::bitflags! {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct Type: usize {
            const EXTRACT = 0x0001;
            const INJECT = 0x0002;
        }
    }

    impl Unset for Type {
        const UNSET: Self = Self::empty();

        fn is_unset(&self) -> bool {
            self.is_empty()
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
    let span_attrs = conf.span_attrs();
    if span_attrs.is_null() {
        span_attrs.init(cf.pool(), 4).ok_or(Error::OutOfMemory)?;
    }

    let (name, value) = cf
        .args()
        .get(1)
        .zip(cf.args().get(2))
        .ok_or_else(|| anyhow!("missing arguments"))?;

    let v = script::ComplexValueCompiler::new(cf)
        .compile(value)
        .map_err(|_| anyhow!("failed to compile complex value"))?;

    span_attrs
        .push(SpanAttr {
            name: name.clone(),
            value: v,
        })
        .ok_or(Error::OutOfMemory)?;

    Ok(())
}

#[native_handler(name = otel_request_start)]
fn on_request_start(req: &RequestRef) -> Result<(), Code> {
    if req.internal() {
        return Err(Code::DECLINED);
    }

    let lcf = Otel::loc_conf(req);

    let sampled = if let Some(v) = lcf.trace {
        let trace = v.evaluate(req).map_err(|_| Code::ERROR)?;

        trace == "on" || trace == "1"
    } else {
        false
    };

    if lcf.trace_ctx.is_empty() && !sampled {
        return Err(Code::DECLINED);
    }

    let ctx = OtelContext::ensure(req).ok_or(Code::ERROR)?;

    ctx.current.sampled = sampled;

    if lcf.trace_ctx.contains(propagation::Type::INJECT) {
        ctx.current.inject(req);
    }

    Ok(())
}

#[native_handler(name = otel_request_end)]
fn on_request_end(req: &RequestRef) -> Result<(), Code> {
    let ctx = OtelContext::ensure(req).ok_or(Code::DECLINED)?;

    Ok(())
}
