# Adding Support for a Board

This document serves as a guide as to what is currently needed for adding support
for a board/device to Ariel OS.

Feel free to report anything that is unclear or missing!

> [!NOTE]
> This guide requires working on your own copy of Ariel OS.
> You may want to fork the repository to easily upstream your changes later.

## Checklist

- Ensure that the HAL [is supported in `ariel-os-hal`][adding-hal-support-book].
- Ensure that the chip [is supported][adding-mcu-support-book].
- [Create a new SBD file](#creating-an-sbd-file) `boards/<your-board-name>.yaml`.
- In `doc/support_matrix.yml`:
  - Add an entry under `builders`.
  - Add an entry under `boards`, that references that new builder.
    Include a link to a `web.archive.org` snapshot that describes the board.
  - Update the generated support pages from the book using `laze build update-book`.

> [!TIP]
> To build every example and test for a board the following command can be used (as this is only for compilation the credentials do not need to be valid):
>
> ```sh
> CONFIG_WIFI_NETWORK='test' CONFIG_WIFI_PASSWORD='password' laze build --global -b <builder>
> ```

## Creating an SBD file

### Boards and SBD Files

Ariel OS uses [SBD files][sbd-book] to describe boards.
Each file corresponds to a physical board.
Boards having multiple MCUs, or MCUs with multiple compilation targets, still require only a single file, but multiple `targets` entries are used to describe the different compilation targets.
When adding support for a new board, it is usually best to copy and adapt an existing one instead of starting from scratch.

### Schema and Guidelines for SBD Items

The SBD schema is defined and documented by the [`sbd-gen-schema` package][sbd-gen-schema-docsrs].
The following sections provide guidelines for each item class, to follow when writing an SBD file to be added in tree.

#### General Guidelines

##### Item Names and Aliases

Many SBD items result in generated identifiers that follow naming schemes similar to `itemX` or `Items` (depending on whether the identifier is intended for a field or a type respectively), where `X` is a 0-based index corresponding to the item's position in the sequence of items of that type.

On top of these automatically generated identifiers, aliases can be manually specified in the SBD file, that will result in additional identifiers for these items.
Such aliases should be added for all items that support them: aliases should follow the silkscreen labels, the schematic symbol annotations, and the labels on the board enclosure (if any).
For example, a button labeled "SW1" on the schematic and "BOOT" on the silkscreen would have both of these as aliases.
Some classes of items can also specify additional aliases that they require.

These aliases are used to easily refer to specific items in applications, to provide identifiers that do not depend on their ordering in the SBD file, and to allow application portability between similar boards (by attaching additional semantics to individual items).
The case of these alias identifiers is preserved.

#### SBD Item Classes

##### `chip`

The board's chip, needs to correspond to an existing laze context listed in `boards/ariel-chips.yaml`.

##### `leds`

The LEDs present on the board, that can be controlled by the MCU.
Currently, each color channel of a multicolor LED is modeled as an individual LED.
Addressable LEDs are not part of this item class.

##### `buttons`

Push-buttons present on the board.
Push-buttons are typically monostable tactile switches.

##### `switches`

Switches present on the board.
Push-buttons qualify as switches, but it is generally preferable to add them as `buttons`.

##### `i2c`

I2C buses present on the board.
To qualify, there should be at least either:

- I2C nodes on the board,
- A pair of pull-up resistors strong enough for I2C operation on the board (even if the lines are only exposed on connectors without other nodes on the board),
- A connector whose pins are dedicated to I2C (e.g., Grove, STEMMA QT, Qwiic, Arduino Modulino, mikroBUS),
- A well-known connector/header/pinout that conventionally includes I2C pins (e.g., Arduino headers, Seeed Studio XIAO pinout).

Some aliases should use the following conventions:

- `ArduinoI2c`: The I2C bus is exposed on Arduino headers.
- `QwiicI2c`: The I2C bus is exposed on a Qwiic connector.

##### `uarts`

UART RX/TX present on the board.
Currently only host-facing UARTs should be added.
Host-facing UARTs are UARTs exposed through a USB ⟷ UART adapter present on the board (or an interface MCU or equivalent acting as such adapter).

### Extra Steps for Some MCU Families

#### `stm32`

- STM32 chips do not have a dedicated SWI, so you need to choose one. Select any unused interrupt, typically that of the WWDG, and set the `targets.<board_name>.ariel.swi` field in the SBD file.

### Regenerating the Board Package

With the SBD file in place, regenerate the `ariel-os-boards` crate.
To do that, install [`sbd-gen`][sbd] with `cargo install sbd-gen`, then run the following command from the `ariel os` repository root:

```sh
sbd-gen generate-ariel boards -o src/ariel-os-boards --mode update
```

[sbd]: https://github.com/ariel-os/sbd
[sbd-gen-schema-docsrs]: https://docs.rs/sbd-gen-schema/latest/sbd_gen_schema/
[sbd-book]: ./structured-board-descriptions.md
[ci-builder-lists]: https://github.com/ariel-os/ariel-os/blob/5c23c04cd51c5cfebd561c9d27a47d4e26cdb649/.github/workflows/build.yml#L147-L168
[i2c-pull-up-resistors-book]: ./i2c.md#configuring-other-settings
[adding-hal-support-book]: ./adding-hal-support.md#adding-support-for-an-embassy-halmcu-family
[adding-mcu-support-book]: ./adding-hal-support.md#adding-support-for-an-mcu-from-a-supported-mcu-family
