# Adding Support for a Board

This document serves as a guide as to what is currently needed for adding support
for a board/device to Ariel OS.

Feel free to report anything that is unclear or missing!

> This guide requires working on your own copy of Ariel OS.
> You may want to fork the repository to easily upstream your changes later.

## Adding Support for a Board

The more similar a board is to one that is already supported, the easier.
It is usually best to copy and adapt an existing one.

- Ensure that the HAL [is supported in `ariel-os-hal`](#adding-support-for-an-embassy-halmcu-family).
- In `laze-project.yml`:
  - `parent`: The MCU's laze context.
  - If the MCU does not have a dedicated software interrupt (SWI), choose one
    now and set the `CONFIG_SWI` environment variable.
  - Ensure there is a way to flash the board:
    - If the MCU is supported by probe-rs, specify `PROBE_RS_CHIP`
      and `PROBE_RS_PROTOCOL`.
    - If the board is based on `esp`, it should inherit the espflash support.
    - If neither of these are supported, please open an issue.
  - Add a builder for the actual board that uses the context from above as `parent`.

Whether to add an intermediate context or just a builder depends on whether the
MCU-specific code can be re-used.

Example for the `st-nucleo-f401re` board:

```yaml
contexts:
  # ...
  - name: stm32f401retx
    parent: stm32
    selects:
      - thumbv7em-none-eabi # actually eabihf, but ariel-os doesn't support hard float yet
    env:
      PROBE_RS_CHIP: STM32F401RETx
      PROBE_RS_PROTOCOL: swd
      RUSTFLAGS:
        - --cfg context=\"stm32f401retx\"
      CARGO_ENV:
        - CONFIG_SWI=USART2

builders:
  # ...
  - name: st-nucleo-f401re
    parent: stm32f401retx
```

- `src/ariel-os-boards/$BOARD`: Add a crate that matches the board's name.
  - This crate should inject the board-specific dependencies to the HAL crates.
- `src/ariel-os-boards`:
  - `Cargo.toml`: Add a Cargo feature that matches the board's name.
  - `src/lib.rs`: Add the board to the dispatch logic.

## Adding Support for an MCU from a Supported MCU family

- In `laze-project.yml`:
  - Add a context for the MCU (if it does not already exist).
    - `parent`: The closest Embassy HAL's context.
    - `selects`: A [rustc-target](#adding-support-for-a-processor-architecture) module.
    - Some environment variables need to be provided:
      - `PROBE_RS_CHIP`: The list of chips can [be found on probe-rs website](https://probe.rs/targets/)

MCU-specific items inside Ariel OS crates are gated behind
`#[cfg(context = $CONTEXT)]` attributes, where `$CONTEXT` is the [MCU's `laze
context` name](./build_system.md#laze-contexts).
These need to be expanded for adding support for the new MCU.

At least the following crates may need to be updated:

- The Ariel OS HAL crate for the MCU family.
- `ariel-os-storage`
- `ariel-os-embassy`

It may also be needed to introduce a new crate in `src/ariel-os-chips`.

> The `ariel-os-chips` crate should eventually not be needed anymore.

## Adding Support for an Embassy HAL/MCU family

As of this writing, Ariel OS supports most HALs that Embassy supports,
including `esp-hal`, `nrf`, `rp`, and `stm32`, but excluding `std` and `wasm`.

The steps to add support for another Embassy supported HAL are:

- `src/ariel-os-hal`:
  - `Cargo.toml`: Add a dependency on the Embassy HAL crate.
  - `src/lib.rs`: Add the Ariel OS HAL to the dispatch logic.
- Create a new Ariel OS HAL crate (similar to `ariel-os-nrf`).

## Adding Support for a Processor Architecture

Each rustc target needs its own module in `laze-project.yml`.
If the processor architecture that is being added is not listed yet, you will
need to take care of that.

Example:

```yaml
modules:
  # ...
  - name: thumbv6m-none-eabi
    depends:
      - cortex-m
    env:
      global:
        RUSTC_TARGET: thumbv6m-none-eabi
        CARGO_TARGET_PREFIX: CARGO_TARGET_THUMBV6M_NONE_EABI
        RUSTFLAGS:
          - --cfg armv6m
```

The variables `RUSTC_TARGET` and `CARGO_TARGET_PREFIX` need to be adjusted.
Add `--cfg $HAL` as needed.

Chances are that if you need to add this, you will also have to add support for
the processor architecture to `ariel-os-bench`, `ariel-os-rt`, `ariel-os-threads`.
