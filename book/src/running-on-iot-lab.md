# Running on IoT-LAB

> This guide has been written for [Ariel OS v0.2.0][ariel-os-tag]; it may or may not work with other versions.
> Going through at least the [Getting Started][getting-started-book] and [Build System][build-system-book] manual pages is required before running firmware on IoT-LAB.

## About IoT-LAB

[IoT-LAB][iot-lab-homepage] is an IoT-oriented testbed—a cluster of development boards which can be used through the network for research and experiments.
After [creating an account][iot-lab-login], your firmware can be uploaded and flashed onto the board(s) of your selection.

IoT-LAB provides two means of interacting with it: through its REST API, or [through the `iotlabcli`][iot-lab-cli] Python package.
`iotlabcli` is intended to be used either directly on your development machine, [with SSH][iot-lab-ssh] through a [gateway server][iot-lab-design-docs], or [using Jupyter notebooks][iot-lab-labs].
In this guide, we are only concerned with using `iotlabcli` directly.

### Boards Supported on IoT-LAB

The following boards present on IoT-LAB are currently supported by Ariel OS:

- [nRF52840-DK](https://www.iot-lab.info/docs/boards/nordic-nrf52840dk/)
- [nRF52-DK](https://www.iot-lab.info/docs/boards/nordic-nrf52dk/)
- [ST B-L475E-IOT01A](https://www.iot-lab.info/docs/boards/st-b-l475e-iot01a/)

Refer to these pages to know the `board-id` of these boards and on which `site` they are deployed.

## Running Firmware on IoT-LAB

In this section, we present how to compile an Ariel OS example, run it on a board on IoT-LAB, and observe its debug output.

The first step is to install [`iotlabcli`][pypi-iotlabcli] on your development machine; this package provides multiple commands to work with IoT-LAB.
Then, authenticate using `iotlab-auth` so that other `iotlab-*` commands know about [your account][iot-lab-login].

> `iotlab-auth` will create an `.iotlabrc` file in your home directory.

### Registering an Experiment

IoT-LAB is based upon the concept of *experiment*, which is a way of synchronizing multiple boards to run experiments—e.g., networking experiments—only when all the required boards are available.

We will not need this kind of synchronization in this guide, but we will still need to register an experiment to get our firmware to run on a board.
This can be done with the following:

```sh
iotlab-experiment submit -n <experiment-name> -d <experiment-duration> -l <site>,<board-id>,<node-id> && iotlab-experiment wait
```

- `<experiment-name>` is a name you need to choose.
- `<site>`, `<board-id>` select which board to run on; see above to know which boards are supported.
- `<node-id>` is an integer refering to an instance of a board, which must be available to run the experiment.
- `<experiment-duration>` is in minutes, and may be set to `1`.
- `iotlab-experiment wait` will block until the requested node is available.
  You may need to use the following command to see which nodes are already being used:

  ```sh
  iotlab-status --experiments-running
  ```

> We assume that you always have at most one experiment running at all time.

### Compiling and Running

Compiling for IoT-LAB does not require anything specific, except enabling the debug output over UART.
The following compiles the `log` example as appropriate (this command must be run in a clone of the Ariel OS repository):

```sh
laze -C examples/log build -s debug-uart -b nrf52840dk -v
```

> The `debug-uart` laze module is currently undocumented in Ariel OS and its behavior and name may change at any time.
> The board-specific UART configuration required for IoT-LAB is currently hard-coded, and will likely require additional configuration in the future.

The above command results in an ELF file (whose path can be read from its output), in `build/bin`.
To upload, flash, and run it on the board selected previously, use the following command (the `-l` parameter must match the one from above):

```sh
iotlab-node -l <site>,<board-id>,<node-id> --flash <elf-file-path>
```

> This command can be chained with the previous one using `&&`, so that registering an experiment and running it is a single operation.

### Observing the Debug Output

When an experiment is actively running, the debug output can be obtained in real time in the web interface: click on a running experiment on [your dashboard][iot-lab-dashboard], and then click on the terminal icon on individual nodes.

It is additionally possible to [use SSH tunneling][iot-lab-ssh-tunneling] to pipe the debug output to your development machine.

[ariel-os-tag]: https://github.com/ariel-os/ariel-os/releases/tag/v0.2.0
[getting-started-book]: https://ariel-os.github.io/ariel-os/dev/docs/book/getting-started.html
[build-system-book]: https://ariel-os.github.io/ariel-os/dev/docs/book/build-system.html
[iot-lab-homepage]: https://www.iot-lab.info/
[iot-lab-login]: https://www.iot-lab.info/testbed/login
[iot-lab-rest-api]: https://www.iot-lab.info/docs/tools/api/
[iot-lab-cli]: https://www.iot-lab.info/docs/tools/cli/
[iot-lab-ssh]: https://www.iot-lab.info/docs/tools/ssh-cli/
[iot-lab-design-docs]: https://www.iot-lab.info/docs/getting-started/design/
[iot-lab-labs]: https://labs.iot-lab.info/
[iot-lab-dashboard]: https://www.iot-lab.info/testbed/dashboard
[iot-lab-ssh-tunneling]: https://www.iot-lab.info/docs/tools/serial-aggregator/#run-serial-aggregator-on-your-computer
[pypi-iotlabcli]: https://pypi.org/project/iotlabcli/
