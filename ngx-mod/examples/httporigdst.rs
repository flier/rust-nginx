#![crate_type = "dylib"]
#![cfg(not(feature = "static-link"))]

use std::{mem::zeroed, net::SocketAddrV4, os::fd::AsRawFd, ptr::NonNull};

use socket2::SockAddr;

use ngx_mod::{
    http::{self, Module as _},
    rt::{
        core::{Code, ConfRef, PoolRef, Str},
        http::{ModuleContext, RequestRef, ValueRef},
        http_debug, native_handler, ngx_var, notice,
    },
    Module, ModuleMetadata,
};

#[derive(Module)]
#[module(name = ngx_http_orig_dst, type = http)]
struct OrigDst;

impl Module for OrigDst {}

impl http::Module for OrigDst {
    type Error = ();
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ();

    fn preconfiguration(cf: &ConfRef) -> Result<(), Code> {
        notice!(cf, "httporigdst: init module");

        cf.add_variables([
            ngx_var!("server_orig_addr", get = ngx_http_orig_dst_addr_variable),
            ngx_var!("server_orig_port", get = ngx_http_orig_dst_port_variable),
        ])
        .map_err(|_| Code::ERROR)
    }
}

fn get_origdst(req: &RequestRef) -> Result<SocketAddrV4, Code> {
    let conn = req.connection();

    if !conn.ty().is_stream() {
        http_debug!(req, "httporigdst: connection is not type SOCK_STREAM");

        return Err(Code::DECLINED);
    }

    let local = conn.local().ok_or_else(|| {
        http_debug!(req, "httporigdst: no local sockaddr from connection");

        Code::ERROR
    })?;

    if !local.is_ipv4() {
        http_debug!(req, "httporigdst: only support IPv4");

        return Err(Code::DECLINED);
    }

    unsafe {
        let mut ss: libc::sockaddr_storage = zeroed();
        let mut len: libc::socklen_t = std::mem::size_of_val(&ss) as libc::socklen_t;

        if libc::getsockopt(
            conn.as_raw_fd(),
            libc::SOL_IP,
            libc::SO_ORIGINAL_DST,
            &mut ss as *mut _ as *mut _,
            &mut len as *mut _,
        ) < 0
        {
            http_debug!(req, "httporigdst: getsockopt failed");

            return Err(Code::DECLINED);
        }

        SockAddr::new(ss, len).as_socket_ipv4().ok_or(Code::ERROR)
    }
}

#[native_handler(name = ngx_http_orig_dst_addr_variable)]
fn server_orig_addr(req: &RequestRef, val: &mut ValueRef, _data: usize) -> Result<(), Code> {
    if let Some(ctx) = OrigDst::module_ctx::<_, OrigDstCtx>(req) {
        http_debug!(req, "httporigdst: found context and binding variable");

        ctx.bind_addr(val);
    } else {
        http_debug!(req, "httporigdst: context not found, getting address");

        let addr = get_origdst(req)?;

        if let Some(ctx) = req.pool().allocate_default::<OrigDstCtx>() {
            http_debug!(req, "httporigdst: saving addr: {}", addr);

            ctx.save(req.pool(), addr)?.bind_addr(val);

            req.set_module_ctx(OrigDst::module(), ctx)
        }
    }

    Ok(())
}

#[native_handler(name = ngx_http_orig_dst_port_variable)]
fn server_orig_port(req: &RequestRef, val: &mut ValueRef, _data: usize) -> Result<(), Code> {
    if let Some(ctx) = OrigDst::module_ctx::<_, OrigDstCtx>(req) {
        http_debug!(req, "httporigdst: found context and binding variable");

        ctx.bind_port(val);
    } else {
        http_debug!(req, "httporigdst: context not found, getting address");

        let addr = get_origdst(req)?;

        if let Some(ctx) = req.pool().allocate_default::<OrigDstCtx>() {
            http_debug!(req, "httporigdst: saving addr: {}", addr);

            ctx.save(req.pool(), addr)?.bind_port(val);

            req.set_module_ctx(OrigDst::module(), ctx)
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Default)]
struct OrigDstCtx {
    orig_dst_addr: Str,
    orig_dst_port: Str,
}

impl OrigDstCtx {
    pub fn save(&mut self, p: &PoolRef, addr: SocketAddrV4) -> Result<&Self, Code> {
        self.orig_dst_addr = p.strdup(addr.ip().to_string()).ok_or(Code::ERROR)?;
        self.orig_dst_port = p.strdup(addr.port().to_string()).ok_or(Code::ERROR)?;

        Ok(self)
    }

    pub fn bind_addr(&self, v: &mut ValueRef) {
        if self.orig_dst_addr.is_empty() {
            v.set_not_found(true);
        } else {
            v.set_valid(true)
                .set_no_cacheable(true)
                .set_not_found(false)
                .set_len(self.orig_dst_addr.len() as u32)
                .set_data(NonNull::new(self.orig_dst_addr.as_ptr() as *mut u8));
        }
    }

    pub fn bind_port(&self, v: &mut ValueRef) {
        if self.orig_dst_port.is_empty() {
            v.set_not_found(true);
        } else {
            v.set_valid(true)
                .set_no_cacheable(true)
                .set_not_found(false)
                .set_len(self.orig_dst_port.len() as u32)
                .set_data(NonNull::new(self.orig_dst_port.as_ptr() as *mut u8));
        }
    }
}
