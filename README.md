# pcm4104

Platform-agnostic Rust driver for the [Texas Instruments PCM4104 4-channel audio DAC](https://www.ti.com/product/PCM4104).

The driver is based on [embedded-hal-async](https://crates.io/crates/embedded-hal-async) and allows the chip configuration via SPI.

## Prerequisites

- The HAL must implement the [embedded-hal-async traits](https://docs.rs/embedded-hal-async/latest/embedded_hal_async/spi/trait.SpiDevice.html). An example for a compatible HAL is [embassy](http://embassy.dev).
- The DAC must be wired for software-controlled mode by setting the `MODE` pin high.
- A minimum of 1024 system clock cycles need to be applied to the `SCK` pin to initialize the chip after power-on or reset. Otherwise, the internal registers will be reset to their default values again after this number of clock cycles have been elapsed.

## Usage

The following example shows the configuration of the DAC using [embassy](http://embassy.dev) on an RP2350.

```rust ignore
use embassy_futures::block_on;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{self, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use pcm4104::*;
use static_cell::StaticCell;

// Alias for convenience.
type Spi1Bus = Mutex<NoopRawMutex, Spi<'static, SPI1, spi::Async>>;

// Get the peripherals.
let p = embassy_rp::init(Default::default());

// Setup a shared SPI bus, so multiple chips can be managed via their CS pins.
let spi_cfg = spi::Config::default();
let spi = Spi::new(p.SPI1, p.PIN_10, p.PIN_11, p.PIN_12, p.DMA_CH0, p.DMA_CH1, spi_cfg);
static SPI_BUS: StaticCell<Spi1Bus> = StaticCell::new();
let spi_bus = SPI_BUS.init(Mutex::new(spi));

// Chip select pin for the DAC.
let cs_pin = Output::new(p.PIN_0, Level::High);

// Instantiate the driver.
let spi_device = SpiDevice::new(spi_bus, cs_pin);
let mut pcm4104 = Pcm4104::new(spi_device);

// Configure the chip for TDM 4-channel use. Apply other options as needed.
// `block_on` is used here exemplary to show how to use the driver in blocking mode.
// That can be handy on startup when the executor is not running yet.
let config = Pcm4104Config {
    audio_data_format: AudioDataFormat::TdmOneBckDelay,
    ..Default::default()
};
block_on(pcm4104.configure(config)).expect("Configuration error");

// Here, a channel is muted in an async context.
pcm4104.set_mute(OutputChannel::Channel3, true).await.expect("Mute error");
```

## License

Published under the MIT license. Any contribution to this project must be provided under the same license conditions.

Author: Oliver Rockstedt <info@sourcebox.de>
