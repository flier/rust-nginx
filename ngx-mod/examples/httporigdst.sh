#!/bin/sh

# Redirect traffic outbound.

iptables -t nat -N NGINX_REDIRECT
iptables -t nat -A NGINX_REDIRECT -p tcp -j REDIRECT --to-port 15502 --dport 15002
iptables -t nat -N NGINX_OUTPUT
iptables -t nat -A OUTPUT -p tcp -j NGINX_OUTPUT
iptables -t nat -A NGINX_OUTPUT -j NGINX_REDIRECT

# Redirect traffic inbound.

iptables -t nat -N NGINX_IN_REDIRECT
iptables -t nat -A NGINX_IN_REDIRECT -p tcp -j REDIRECT --to-port 15502 --dport 15002
iptables -t nat -N NGINX_INBOUND
iptables -t nat -A PREROUTING -p tcp -j NGINX_INBOUND
iptables -t nat -A NGINX_INBOUND -p tcp -j NGINX_IN_REDIRECT
