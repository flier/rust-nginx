#!/bin/sh

nginx_dir=$1
build_dir=$2

bindgen \
    --allowlist-type "^(NGX|ngx)_.*$" \
    --allowlist-function "^(NGX|ngx)_.*$" \
    --allowlist-var "^(NGX|ngx|NGINX|nginx)_.*$" \
    --impl-debug \
    --impl-partialeq \
    --with-derive-default \
    --with-derive-partialeq \
    --output src/bindings.rs \
    nginx.h \
    -- \
    -I$nginx_dir \
    -I$nginx_dir/src/core \
    -I$nginx_dir/src/os/unix \
    -I$nginx_dir/src/event -D NGX_EVENT \
    -I$nginx_dir/src/http -I$nginx_dir/src/http/modules -I$nginx_dir/src/http/v2 -D NGX_HTTP \
    -I$nginx_dir/src/stream -D NGX_STREAM \
    -I$nginx_dir/src/mail -D NGX_MAIL \
    -I$nginx_dir/objs \
    -I$build_dir
