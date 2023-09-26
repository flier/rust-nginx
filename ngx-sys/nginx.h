#include <nginx.h>
#include <ngx_core.h>

#ifdef NGX_MODULE
#include <ngx_module.h>
#endif

#ifdef NGX_EVENT
#include <ngx_event.h>
#endif

#ifdef NGX_HTTP
#include <ngx_http.h>
#endif

#ifdef NGX_MAIL
#include <ngx_mail.h>
#endif

#ifdef NGX_STREAM
#include <ngx_stream.h>
#endif

const size_t NGX_RS_HTTP_MAIN_CONF_OFFSET = NGX_HTTP_MAIN_CONF_OFFSET;
const size_t NGX_RS_HTTP_SRV_CONF_OFFSET = NGX_HTTP_SRV_CONF_OFFSET;
const size_t NGX_RS_HTTP_LOC_CONF_OFFSET = NGX_HTTP_LOC_CONF_OFFSET;

const char *NGX_RS_MODULE_SIGNATURE = NGX_MODULE_SIGNATURE;
