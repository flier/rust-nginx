#include <nginx.h>
#include <ngx_core.h>

#ifdef NGX_MODULE
#include <ngx_module.h>

const char *NGX_RS_MODULE_SIGNATURE = NGX_MODULE_SIGNATURE;
#endif

#ifdef NGX_EVENT
#include <ngx_event.h>
#endif

#ifdef NGX_HTTP
#include <ngx_http.h>

const size_t NGX_RS_HTTP_MAIN_CONF_OFFSET = NGX_HTTP_MAIN_CONF_OFFSET;
const size_t NGX_RS_HTTP_SRV_CONF_OFFSET = NGX_HTTP_SRV_CONF_OFFSET;
const size_t NGX_RS_HTTP_LOC_CONF_OFFSET = NGX_HTTP_LOC_CONF_OFFSET;
#endif

#ifdef NGX_MAIL
#include <ngx_mail.h>

const size_t NGX_RS_MAIL_MAIN_CONF_OFFSET = NGX_MAIL_MAIN_CONF_OFFSET;
const size_t NGX_RS_MAIL_SRV_CONF_OFFSET = NGX_MAIL_SRV_CONF_OFFSET;
#endif

#ifdef NGX_STREAM
#include <ngx_stream.h>

const size_t NGX_RS_STREAM_MAIN_CONF_OFFSET = NGX_STREAM_MAIN_CONF_OFFSET;
const size_t NGX_RS_STREAM_SRV_CONF_OFFSET = NGX_STREAM_SRV_CONF_OFFSET;
#endif
