use std::{ops::Deref, ptr::NonNull, slice};

use bitflags::bitflags;
use foreign_types::foreign_type;
use num_enum::FromPrimitive;

use crate::{
    core::{BufRef, ConnRef, LogRef, PoolRef},
    ffi, flag,
    http::{upstream::UpstreamRef, UnsafeLocConf, UnsafeMainConf, UnsafeSrvConf},
    native_callback, never_drop, property, str, AsRawRef, Error,
};

use super::{body::BodyRef, HeadersInRef, HeadersOutRef, Method};

foreign_type! {
    pub unsafe type Request: Send {
        type CType = ffi::ngx_http_request_t;

        fn drop = never_drop::<ffi::ngx_http_request_t>;
    }
}

impl Deref for RequestRef {
    type Target = HeadersInRef;

    fn deref(&self) -> &Self::Target {
        self.headers_in()
    }
}

impl UnsafeMainConf for RequestRef {
    unsafe fn unchecked_main_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().main_conf.add(idx).read().cast())
    }
}

impl UnsafeSrvConf for RequestRef {
    unsafe fn unchecked_srv_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().srv_conf.add(idx).read().cast())
    }
}

impl UnsafeLocConf for RequestRef {
    unsafe fn unchecked_loc_conf<T>(&self, idx: usize) -> Option<NonNull<T>> {
        NonNull::new(self.as_raw().loc_conf.add(idx).read().cast())
    }
}

impl RequestRef {
    property! {
        /// client connection
        connection: &ConnRef;

        /// Request upstream object for proxying.
        upstream as &mut UpstreamRef;

        /// Request pool.
        pool: &PoolRef;

        /// Buffer into which the client HTTP request header is read.
        header_in: &BufRef;

        /// Input HTTP headers objects.
        &headers_in: &HeadersInRef;

        /// Output HTTP headers objects.
        &headers_out: &HeadersOutRef;

        /// Client request body object.
        request_body as &BodyRef;

        start_sec: i64;
        start_msec: usize;

        /// Client HTTP protocol version in numeric form
        http_version: usize;

        /// Client HTTP protocol major version in numeric
        http_minor(): u32;

        /// Client HTTP protocol minor version in numeric
        http_major(): u32;

        /// the main request object.
        main as &Self;

        /// the parent request of a subrequest.
        parent as &Self;

        header_size: usize;

        request_length: i64;

        err_status: usize;

        /// Request reference counter.
        count(): u32;

        /// Current subrequest nesting level.
        subrequests(): u32;

        /// Counter of blocks held on the request.
        blocked(): u32;

        uri_changes(): u32;

        request_body_file_log_level(): u32;

        limit_conn_status(): u32;
        limit_req_status(): u32;
    }

    str! {
        /// Request line in the original client request.
        &request_line;

        /// the name of client HTTP request method.
        &method_name;

        /// client HTTP protocol version in its original text form
        &http_protocol;

        /// URI for the current request.
        &uri;

        /// arguments for the current request.
        &args?;

        /// file extension for the current request.
        &exten?;

        /// URI in the original client request.
        &unparsed_uri;

        /// HTTP request schema.
        &schema?;
    }

    callback! {
        read_event_handler: EventHandlerFn;
        write_event_handler: EventHandlerFn;
        content_handler: HandlerFn;
    }

    flag! {
        aio;

        /// URI with "/." and on Win32 with "//"
        complex_uri;

        /// URI with "%"
        quoted_uri;

        /// URI with "+"
        plus_in_uri;

        /// URI with empty path
        empty_path_in_uri;

        invalid_header;

        add_uri_to_alias;
        valid_location;
        valid_unparsed_uri;
        uri_changed;

        request_body_in_single_buf;
        request_body_in_file_only;
        request_body_in_persistent_file;
        request_body_in_clean_file;
        request_body_file_group_access;
        request_body_no_buffering;

        subrequest_in_memory;
        waited;

        proxy;
        bypass_cache;
        no_cache;

        limit_rate_set;
        limit_rate_after_set;

        pipeline;
        chunked;

        /// the output does not require a body.
        header_only;

        expect_trailers;

        /// whether client connection keepalive is supported.
        keepalive;

        lingering_close;
        discard_body;
        reading_body;

        /// the current request is internal.
        internal;

        error_page;
        filter_finalize;
        post_action;
        request_complete;
        request_output;

        /// the output header has already been sent by the request.
        header_sent;

        expect_tested;
        root_tested;
        done;
        logged;

        /// the output produced in memory buffers rather than files.
        main_filter_need_in_memory;

        /// the output produced in memory buffers rather than files.
        filter_need_in_memory;

        /// the request output be produced in temporary buffers, but not in readonly memory buffers or file buffers.
        filter_need_temporary;

        preserve_body;

        /// a partial response can be sent to the client, as requested by the HTTP Range header.
        allow_ranges;

        /// a partial response can be sent while a subrequest is being processed.
        subrequest_ranges;

        /// only a single continuous range of output data can be sent to the client.
        single_range;

        disable_not_modified;
        stat_reading;
        stat_writing;
        stat_processing;

        background;
        health_check;
    }

    /// the client HTTP request method.
    pub fn method(&self) -> Method {
        Method::from_bits_truncate(unsafe { self.as_raw().method as u32 })
    }

    pub fn as_method(&self) -> Option<http::Method> {
        self.method().try_into().ok().or_else(|| {
            http::Method::from_bytes(unsafe {
                let r = self.as_raw();

                slice::from_raw_parts(
                    r.request_start,
                    r.method_end.offset_from(r.request_start) as usize,
                )
            })
            .ok()
        })
    }

    pub fn version(&self) -> (u32, u32) {
        unsafe {
            let v = self.as_raw().http_version;

            ((v >> 16) as u32, (v & 0xFFFF) as u32)
        }
    }

    pub fn http_state(&self) -> State {
        unsafe { State::from(self.as_raw().http_state()) }
    }

    /// Bitmask showing which modules have buffered the output produced by the request.
    pub fn buffered(&self) -> Buffered {
        unsafe { Buffered::from_bits_truncate(self.as_raw().buffered()) }
    }
}

impl AsRef<LogRef> for RequestRef {
    fn as_ref(&self) -> &LogRef {
        self.connection().log()
    }
}

#[native_callback]
pub type HandlerFn = fn(req: &RequestRef) -> Result<(), Error>;

#[native_callback]
pub type EventHandlerFn = fn(req: &RequestRef);

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum State {
    #[default]
    InitingRequest = ffi::ngx_http_state_e_NGX_HTTP_INITING_REQUEST_STATE,
    ReadingRequest = ffi::ngx_http_state_e_NGX_HTTP_READING_REQUEST_STATE,
    ProcessRequest = ffi::ngx_http_state_e_NGX_HTTP_PROCESS_REQUEST_STATE,

    ConnectUpstream = ffi::ngx_http_state_e_NGX_HTTP_CONNECT_UPSTREAM_STATE,
    WritingUpstream = ffi::ngx_http_state_e_NGX_HTTP_WRITING_UPSTREAM_STATE,
    ReadingUpstream = ffi::ngx_http_state_e_NGX_HTTP_READING_UPSTREAM_STATE,

    WritingRequest = ffi::ngx_http_state_e_NGX_HTTP_WRITING_REQUEST_STATE,
    LingeringClose = ffi::ngx_http_state_e_NGX_HTTP_LINGERING_CLOSE_STATE,
    Keepalive = ffi::ngx_http_state_e_NGX_HTTP_KEEPALIVE_STATE,
}

bitflags! {
    pub struct Buffered: u32 {
        const LOWLEVEL = ffi::NGX_HTTP_LOWLEVEL_BUFFERED;
        const WRITE = ffi::NGX_HTTP_WRITE_BUFFERED;
        const GZIP = ffi::NGX_HTTP_GZIP_BUFFERED;
        const SSI = ffi::NGX_HTTP_SSI_BUFFERED;
        const SUB = ffi::NGX_HTTP_SUB_BUFFERED;
        const COPY = ffi::NGX_HTTP_COPY_BUFFERED;
    }
}
