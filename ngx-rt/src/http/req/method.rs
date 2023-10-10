use bitflags::bitflags;

use crate::ffi;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Method : u32 {
        const UNKNOWN = ffi::NGX_HTTP_UNKNOWN;
        const GET = ffi::NGX_HTTP_GET;
        const HEAD = ffi::NGX_HTTP_HEAD;
        const POST = ffi::NGX_HTTP_POST;
        const PUT = ffi::NGX_HTTP_PUT;
        const DELETE = ffi::NGX_HTTP_DELETE;
        const MKCOL = ffi::NGX_HTTP_MKCOL;
        const COPY = ffi::NGX_HTTP_COPY;
        const MOVE = ffi::NGX_HTTP_MOVE;
        const OPTIONS = ffi::NGX_HTTP_OPTIONS;
        const PROPFIND = ffi::NGX_HTTP_PROPFIND;
        const PROPPATCH = ffi::NGX_HTTP_PROPPATCH;
        const LOCK = ffi::NGX_HTTP_LOCK;
        const UNLOCK = ffi::NGX_HTTP_UNLOCK;
        const PATCH = ffi::NGX_HTTP_PATCH;
        const TRACE = ffi::NGX_HTTP_TRACE;
        const CONNECT = ffi::NGX_HTTP_CONNECT;
    }
}

impl TryFrom<Method> for ::http::Method {
    type Error = Method;

    fn try_from(m: Method) -> Result<Self, Self::Error> {
        Ok(match m {
            Method::GET => ::http::Method::GET,
            Method::HEAD => ::http::Method::HEAD,
            Method::POST => ::http::Method::POST,
            Method::PUT => ::http::Method::PUT,
            Method::DELETE => ::http::Method::DELETE,
            Method::MKCOL => ::http::Method::from_bytes(b"MKCOL").unwrap(),
            Method::COPY => ::http::Method::from_bytes(b"COPY").unwrap(),
            Method::MOVE => ::http::Method::from_bytes(b"MOVE").unwrap(),
            Method::OPTIONS => ::http::Method::OPTIONS,
            Method::PROPFIND => ::http::Method::from_bytes(b"PROPFIND").unwrap(),
            Method::PROPPATCH => ::http::Method::from_bytes(b"PROPPATCH").unwrap(),
            Method::LOCK => ::http::Method::from_bytes(b"LOCK").unwrap(),
            Method::UNLOCK => ::http::Method::from_bytes(b"UNLOCK").unwrap(),
            Method::PATCH => ::http::Method::PATCH,
            Method::TRACE => ::http::Method::TRACE,
            Method::CONNECT => ::http::Method::CONNECT,
            _ => return Err(m),
        })
    }
}
