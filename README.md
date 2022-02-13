# caster

[![crates.io](https://img.shields.io/crates/d/caster.svg)](https://crates.io/crates/caster)
[![docker pulls](https://img.shields.io/docker/pulls/mihaigalos/caster)](https://hub.docker.com/r/mihaigalos/caster)

Publish stdout via http.

![caster](screenshots/caster.png)

# Why?

`caster` is a command line server for easy exposing of local services to the internet.

It was initially conceived for running diagnostics on remote machines, where `ssh` was either not possible or not preferred.

It's dockerized and offers a very small [footprint](https://contains.dev/mihaigalos/caster), `<10MB`.

# Usage

## Server
```bash
$ docker run --rm -it -p 8080:8080 mihaigalos/caster
```
To avoid i.e. leaking sensitive information such as passwords, you can pass the `--secure` flag. This will only permit access to named endpoints such as `/ping` or `/curl`.

## Client

Test if the remote endpoint can reach the internet:
```bash
$ curl localhost:8080 -XPOST -d 'ping -c 3 google.com'

PING google.com (142.251.36.206): 56 data bytes
64 bytes from 142.251.36.206: seq=0 ttl=117 time=30.143 ms
64 bytes from 142.251.36.206: seq=1 ttl=117 time=19.261 ms
64 bytes from 142.251.36.206: seq=2 ttl=117 time=15.664 ms

--- google.com ping statistics ---
3 packets transmitted, 3 packets received, 0% packet loss
round-trip min/avg/max = 15.664/21.689/30.143 ms
```

Test access, ignore TLS/SSL:
```bash
$ curl localhost:8080 -XPOST -d 'curl -sSLk news.ycombinator.com'

<data>
```

The following are aliases for the above:
```bash
$ curl localhost:8080/ping -XPOST -d '-c 3 google.com'
$ curl localhost:8080/curl -XPOST -d '-sSLk news.ycombinator.com'
```
