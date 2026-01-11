#![doc = include_str!("../README.md")]
#![no_std]

use embedded_hal_async::spi::SpiDevice;

/// Driver for the DAC.
pub struct Pcm4104<SPI> {
    spi: SPI,
}

impl<SPI> Pcm4104<SPI>
where
    SPI: SpiDevice,
{
    /// Returns a new driver.
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Configures the DAC with specific settings.
    pub async fn configure(&mut self, config: Pcm4104Config) -> Result<(), Error> {
        // Check if de-emphasis can be activated.
        if config.de_emphasis != DeEmphasis::Disabled
            && config.sampling_mode != SamplingMode::SingleRate
        {
            return Err(Error::DeEmphasisNotAvailable);
        }

        // Register 5: Function Control Register.
        let mut value = config.de_emphasis as u8 | ((config.output_phase as u8) << 2);
        if config.zero_data_mute {
            value |= 0b1 << 3;
        }
        if config.soft_mute {
            value |= 0b1111 << 4;
        }
        self.write_register(5, value).await?;

        // Register 6: System Control Register.
        let mut value = config.sampling_mode as u8;
        if config.power_down {
            value |= 0b11 << 2;
        }
        self.write_register(6, value).await?;

        // Register 7: Audio Serial Port Control Register.
        let value = config.audio_data_format as u8
            | ((config.lrck_polarity as u8) << 4)
            | ((config.bck_sampling_edge as u8) << 5);
        self.write_register(7, value).await
    }

    /// Sets the digital output attenuation for a channel.
    pub async fn set_attenuation(
        &mut self,
        channel: OutputChannel,
        atten: u8,
    ) -> Result<(), Error> {
        let addr = match channel {
            OutputChannel::Channel1 => 1,
            OutputChannel::Channel2 => 2,
            OutputChannel::Channel3 => 3,
            OutputChannel::Channel4 => 4,
        };
        self.write_register(addr, atten).await
    }

    /// Sets the soft mute on/off for a channel.
    pub async fn set_mute(&mut self, channel: OutputChannel, mute: bool) -> Result<(), Error> {
        let mut value = self.read_register(5).await?;

        let bit_mask = match channel {
            OutputChannel::Channel1 => 0b0001,
            OutputChannel::Channel2 => 0b0010,
            OutputChannel::Channel3 => 0b0100,
            OutputChannel::Channel4 => 0b1000,
        } << 4;

        if mute {
            value |= bit_mask;
        } else {
            value &= !bit_mask;
        }

        self.write_register(5, value).await
    }

    /// Sets the power down state for pair of channels.
    /// The state can only be changed for channel 1/2 or 3/4 at once,
    /// setting one channel of a pair will also affect the other.
    pub async fn set_power_down(
        &mut self,
        channel: OutputChannel,
        down: bool,
    ) -> Result<(), Error> {
        let mut value = self.read_register(6).await?;

        let bit_mask = match channel {
            OutputChannel::Channel1 | OutputChannel::Channel2 => 0b01,
            OutputChannel::Channel3 | OutputChannel::Channel4 => 0b10,
        } << 2;

        if down {
            value |= bit_mask;
        } else {
            value &= !bit_mask;
        }

        self.write_register(6, value).await
    }

    /// Performs a software reset.
    pub async fn reset(&mut self) -> Result<(), Error> {
        self.write_register(6, 0b1000000).await
    }

    /// Reads a single register.
    pub async fn read_register(&mut self, addr: u8) -> Result<u8, Error> {
        if addr > 7 {
            return Err(Error::InvalidRegisterAddress);
        }

        let command = 0b10100000 | addr;
        let tx_buf = [command, 0];
        let mut rx_buf = [0; 2];

        let result = self.spi.transfer(&mut rx_buf, &tx_buf).await;

        result.map(|_| rx_buf[1]).map_err(|_| Error::SpiError)
    }

    /// Writes a single register.
    pub async fn write_register(&mut self, addr: u8, value: u8) -> Result<(), Error> {
        if addr > 7 {
            return Err(Error::InvalidRegisterAddress);
        }

        let command = 0b00100000 | addr;
        let tx_buf = [command, value];

        self.spi.write(&tx_buf).await.map_err(|_| Error::SpiError)
    }
}

/// Driver configuration settings.
#[derive(Debug, Default, Clone)]
pub struct Pcm4104Config {
    /// Sampling mode.
    pub sampling_mode: SamplingMode,

    /// Audio data format.
    pub audio_data_format: AudioDataFormat,

    /// LRCK polarity.
    pub lrck_polarity: LrckPolarity,

    /// BCK sampling edge.
    pub bck_sampling_edge: BckSamplingEdge,

    /// Digital de-emphasis.
    pub de_emphasis: DeEmphasis,

    /// Output phase.
    pub output_phase: OutputPhase,

    /// Zero data mute.
    pub zero_data_mute: bool,

    /// Soft mute.
    pub soft_mute: bool,

    /// Power down.
    pub power_down: bool,
}

/// Sampling mode selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum SamplingMode {
    /// Single Rate.
    #[default]
    SingleRate = 0b00,

    /// Dual Rate.
    DualRate = 0b01,

    /// Quad Rate.
    QuadRate = 0b10,
}

/// Audio data format selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioDataFormat {
    /// 24-bit left justified.
    #[default]
    LeftJustified24Bit = 0b000,

    /// 24-bit I2S.
    I2s24Bit = 0b001,

    /// TDM with zero BCK delay.
    TdmZeroBckDelay = 0b010,

    /// TDM with one BCK delay.
    TdmOneBckDelay = 0b011,

    /// 24-bit right justified.
    RightJustified24Bit = 0b100,

    /// 20-bit right justified.
    RightJustified20Bit = 0b101,

    /// 18-bit right justified.
    RightJustified18Bit = 0b110,

    /// 16-bit right justified.
    RightJustified16Bit = 0b111,
}

/// LRCK polarity selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum LrckPolarity {
    /// Normal polarity.
    #[default]
    Normal = 0b0,

    /// Inverted polarity.
    Inverted = 0b1,
}

/// Bitclock sampling edge selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum BckSamplingEdge {
    /// Rising edge.
    #[default]
    Rising = 0b0,

    /// Falling edge.
    Falling = 0b1,
}

/// De-Emphasis selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DeEmphasis {
    /// Disabled.
    #[default]
    Disabled = 0b00,

    /// Enabled for fs = 48kHz.
    Fs48Khz = 0b01,

    /// Enabled for fs = 44.1kHz.
    Fs44_1Khz = 0b10,

    /// Enabled for fs = 32kHz.
    Fs32Khz = 0b11,
}

/// Output phase selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum OutputPhase {
    /// Non-inverted.
    #[default]
    NonInverted = 0b0,

    /// Inverted.
    Inverted = 0b1,
}

/// Output channel selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputChannel {
    /// Output channel 1.
    Channel1,

    /// Output channel 2.
    Channel2,

    /// Output channel 3.
    Channel3,

    /// Output channel 4.
    Channel4,
}

/// Errors returned by the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// SPI communication error.
    SpiError,

    /// Register address out of range.
    InvalidRegisterAddress,

    /// De-emphasis not available in the selected sampling mode.
    DeEmphasisNotAvailable,
}
