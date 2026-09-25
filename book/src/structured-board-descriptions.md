# Structured Board Descriptions

Ariel OS introduces declarative, structured board descriptions.
These are intended to encode the information of what hardware is present on the board, in a way that is machine processable.
In Ariel OS, they are used to generate the `ariel-os-boards` package.

## SBD files

Structured board description (SBD) files are YAML files that follow the schema defined by the [`sbd-gen-schema` package].
For each *target* of a board, they specify the chip, some OS-specific configuration required for the target, and what hardware is present on the board and is accessible and relevant for the target.
A board/SBD file may comprise multiple targets, e.g., when the board features multiple microcontrollers.
Targets correspond exactly to [Ariel OS's laze builders].
Usable chip names currently are Ariel OS's chip laze contexts.

> [!NOTE]
> The eventual goal is that SBD files be completely OS-agnostic, however this is not entirely the case right now.

Existing SBD files are found in the [`boards/` directory].
See the documentation of [`sbd_gen_schema`] for the meaning of the various constructs.

Currently SBD files cannot be defined out of tree.
See [the Developer Guide] to learn how to add support for a new board.
When adding a new SBD file, or updating an existing one, [`sbd-gen`] is used to re-generate [`ariel-os-boards`] from the SBD files.

## Using the Generated `ariel-os-boards` from Applications

To use the board information provided by `ariel-os-boards`, the package must be added as a dependency of the application, as already done in [the application templates].

TODO
