# pcm4104

Platform-agnostic Rust driver for the [Texas Instruments PCM4104 4-channel audio DAC](https://www.ti.com/product/PCM4104).

The driver is based on [embedded-hal-async](https://crates.io/crates/embedded-hal-async) and allows the chip configuration via SPI.

## Prerequisites

- The HAL must implement the [embedded-hal-async traits](https://docs.rs/embedded-hal-async/latest/embedded_hal_async/spi/trait.SpiDevice.html). An example for a compatible HAL is [embassy](http://embassy.dev).
- The DAC must be wired for software-controlled mode by setting the `MODE` pin high.
- A minimum of 1024 system clock cycles need to be applied to the `SCK` pin to initialize the chip after power-on or reset. Otherwise, the internal registers will be reset to their default values again after this number of clock cycles have been elapsed.

## License

Published under the MIT license. Any contribution to this project must be provided under the same license conditions.

Author: Oliver Rockstedt <info@sourcebox.de>
