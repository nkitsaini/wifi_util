Simple utility to deal with Syrotech brand routers. I've built this to restart syrotech router from a bridge router running openwrt. Project does not include openssl to simplify cross-compilation.

Syrotech routers have IP based login, so if the bridge router logs in, all the clients will automatically log in. This binary by default only logs into the router. And can be configured to also restart the router by passing `-r` arg.   

By the way, if you don't know what I'm talking about most probably you don't need any of this. This is built for very specific reqirement.

# Cross builds
Uses https://github.com/cross-rs/cross for cross-compilation
## Install requirements
```sh
cargo install cross
```
## Build
```sh
# Builds for mipsel architecture (Architecture for tplink C6 v3 router)
./scripts/router_build
```

# Run
```sh
cargo run -- --help
# or from binary
wifi_util --help
```

