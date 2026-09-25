# Sensors

<!-- NOTE: Should be kept semantically in sync with the homepage of `ariel-os-sensors` and `ariel_os::sensors`. -->
Ariel OS introduces an sensor API and a matching ecosystem of sensor drivers.
This sensor API has two main goals:

- Providing a unified way of accessing the readings from all registered sensor driver instances in a homogeneous way.
- Making it easy and as transparent as possible to substitute a specific sensor device by a similar one from the same category.

## Definitions

The sensor API and this documentation rely on the following definitions:

<!-- NOTE: Should be kept semantically in sync with the homepage of `ariel-os-sensors` and `ariel_os::sensors`. -->
- A *sensor device* is a device measuring one or multiple physical quantities and reporting them as one or more digital values—we call these values *samples*.
- Sensor devices measuring the same physical quantity are said to be part of the same *sensor category*.
  A sensor device may be part of multiple sensor categories.
- A *measurement* is the physical operation of measuring one or several physical quantities.
- A *reading* is the digital result returned by a sensor device after carrying out a measurement.
  Samples of different physical quantities can therefore be part of the same reading.
- A *sensor driver* refers to a sensor device as exposed by the sensor abstraction layer.
- A *sensor driver instance* is an instance of a sensor driver.

> [!NOTE]
> Currently, existing sensor drivers are mainly focused on sensor devices with a digital serial interface (e.g., [I2C][i2c-book], [SPI][spi-book]), but this is not a limitation of the sensor API: it should also accommodate analog sensor devices that require readings of the internal ADC for instance.

## Ariel OS Sensor API

The core of the Sensor API is the dyn-compatible `ariel_os_driver::Sensor` trait.
It defines a single interface that sensor drivers implement, to allow homogeneous access to their readings.
The main two methods are [`Sensor::trigger_measurement()`][ariel-os-sensors-mod-sensor-trigger-measurement-rustdoc] and [`Sensor::wait_for_reading()`][ariel-os-sensors-mod-sensor-wait-for-reading-rustdoc] which, together, allow triggering a measurement and awaiting its readings, in a polling fashion: the measurement is explicitly triggered, but [`Sensor::wait_for_reading()`][ariel-os-sensors-mod-sensor-wait-for-reading-rustdoc] returns a `Future`, which completes when readings are available (which may take some time depending on the sensor device).

> [!NOTE]
> In the future, the API should be expanded to provide abstractions over sensor device interrupts.

Additionally, a registry exposes sensor driver instances that have been registered in the application, providing a single access point to them, and allowing for instance to iterate at runtime over all the sensor driver instances registered in the application.

## Using Sensor Drivers through the API

This section covers the usage of existing sensor drivers.
<!-- To implement a new sensor driver, see [the dedicated section]. -->

### Initializing the Sensor Driver

To use a sensor driver implementing Ariel OS Sensor API, the sensor's package must first be added as a dependency.
Then, the sensor driver must be instantiated, and the instance registered into registry.

TODO: example (only link?)

TODO: initializing the driver, runner

TODO: later code-generated from board description files

### Accessing the Sensor Driver Instance

Sensor driver instances are accessed through the registry: [`ariel_os::sensors::REGISTRY`][ariel-os-sensors-mod-registry-rustdoc].
Its [`sensors()`][ariel-os-sensors-mod-registry-sensors-rustdoc] method returns an iterator over all registered sensor driver instances.
If a specific sensor driver instance is desired, the iterator can be filtered based on the sensor categories, obtained with [`Sensor::categories()`][ariel-os-sensors-mod-sensor-categories-rustdoc], and on the sensor driver *instance*'s label, obtained with [`Sensor::label()`][ariel-os-sensors-mod-sensor-label-rustdoc].
The label is defined when instantiating the sensor driver and allows disambiguating between multiple identical sensor devices: the label can for instance indicate where the sensor device is located or what its function is.

### Triggering a Measurement and Obtaining Readings

After obtaining a reference to the sensor driver instance, a measurement can be triggered and then awaited.
There is no bound on how long a measurement can take, so [`Sensor::wait_for_reading()`][ariel-os-sensors-mod-sensor-wait-for-reading-rustdoc] returns a `Future` that completes when readings are ready.
The split between [`Sensor::trigger_measurement()`][ariel-os-sensors-mod-sensor-trigger-measurement-rustdoc] and [`Sensor::wait_for_reading()`][ariel-os-sensors-mod-sensor-wait-for-reading-rustdoc] makes it possible to trigger parallel measurements while having to await the readings of only one sensor driver instance at a time, avoiding the need for allocating multiple `Future`s.
Note that sensor drivers may implement time-based polling instead of relying on sensor device interrupts to determine when readings are available.

TODO: Using the reading channels, accuracy

TODO: Note on floats: the API is designed not to use floats so it can easily be used on the smallest microcontrollers; floats can still be constructed and used, especially for displaying values; or fixed-point numbers can be used

[i2c-book]: ./i2c.md
[spi-book]: ./spi.md
[ariel-os-sensors-mod-registry-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/static.REGISTRY.html
[ariel-os-sensors-mod-registry-sensors-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/registry/struct.Registry.html#method.sensors
[ariel-os-sensors-mod-sensor-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/trait.Sensor.html
[ariel-os-sensors-mod-sensor-trigger-measurement-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/trait.Sensor.html#tymethod.trigger_measurement
[ariel-os-sensors-mod-sensor-wait-for-reading-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/trait.Sensor.html#tymethod.wait_for_reading
[ariel-os-sensors-mod-sensor-categories-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/trait.Sensor.html#tymethod.categories
[ariel-os-sensors-mod-sensor-label-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/sensors/trait.Sensor.html#tymethod.label
