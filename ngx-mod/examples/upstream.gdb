# http://sourceware.org/gdb/wiki/FAQ: to disable the
# "---Type <return> to continue, or q <return> to quit---"
# in batch mode:
set width 0
set height 0
set verbose off
set pagination off
set breakpoint pending on
set follow-fork-mode child

show args

# br ngx_http_upstream_custom
# br ngx_http_upstream_init_custom
# br http_upstream_init_custom_peer
# br ngx_http_upstream_get_custom_peer
# br ngx_http_upstream_free_custom_peer

# br upstream::set_custom
# br upstream::init_custom
# br upstream::init_custom_peer
# br upstream::get_custom_peer
# br upstream::free_custom_peer

# br ngx_http_upstream_init_request
# br ngx_http_upstream_init_round_robin_peer
# br ngx_http_upstream_get_round_robin_peer
# br ngx_http_upstream_free_round_robin_peer

r
