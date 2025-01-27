# Examples

## Overview

This directory contains example applications that showcase how to use Ariel OS.

- [benchmark/](./benchmark): how to use `benchmark()`
- [coap-server](./coap-server) and [coap-client](./coap-client): Application level networking examples
- [device-metadata/](./device-metadata): Retrieve metadata about the running device
- [gpio/](./gpio): GPIO pin control example.
- [hello-world/](./hello-world): a classic, async version
- [hello-world-threading/](./hello-world-threading): a classic, using a thread
- [http-client/](./http-client): HTTP client example
- [http-server/](./http-server): HTTP server example
- [log](./log): Example demonstrating different log levels for printing feedback messages.
- [minimal/](./minimal): minimized to the max Ariel OS config
- [random/](./random): demonstrate obtaining random values
- [tcp-echo/](./tcp-echo): TCP echo example
- [thread-async-interop/](./thread-async-interop): how to make async tasks and preemptively scheduled threads interoperate
- [threading/](./threading): how to start and use preemptively scheduled threads
- [threading-channel/](./threading-channel): how to use `ariel_os::thread::sync::Channel` for passing messages between threads
- [threading-event/](./threading-event): how to use `ariel_os::thread::sync::Event`
- [udp-echo/](./udp-echo): UDP echo example
- [usb-keyboard/](./usb-keyboard): USB HID example
- [usb-serial/](./usb-serial): USB serial example

## Networking

Some examples demonstrate networking capabilities. By default, they will listen on a static
IPv4 address: `10.42.0.61`.
To make the device use a DHCP client for address allocation instead of the static address,
disable the `override-network-config` feature in the example's `Cargo.toml`.

See the [networking documentation](https://ariel-os.github.io/ariel-os/dev/docs/book/networking.html) to learn how to set up networking.
