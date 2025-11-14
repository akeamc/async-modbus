# `async-modbus`

`async-modbus` provides a lightweight implementation of Modbus requests and responses with the help of [`zerocopy`](https://github.com/google/zerocopy). It is designed for resource-constrained environments (being `no_std` by default) like embedded systems but can be used in any Rust project.

There is a basic client implementation available behind the `embedded-io` feature flag (enabled by default).
