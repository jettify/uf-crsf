use super::codec::{Reader, Writer};
use crate::CrsfParsingError;

const SUB_COMMAND_ID_LED_SET_DEFAULT: u8 = 0x01;
const SUB_COMMAND_ID_LED_OVERRIDE_COLOR: u8 = 0x02;
const SUB_COMMAND_ID_LED_OVERRIDE_PULSE: u8 = 0x03;
const SUB_COMMAND_ID_LED_OVERRIDE_BLINK: u8 = 0x04;
const SUB_COMMAND_ID_LED_OVERRIDE_SHIFT: u8 = 0x05;

/// LED HSV color. Hue is 0–359 degrees; saturation and value are 0–100 percent.
///
/// Encoded as three bytes, MSB first: 9 hue bits, 7 saturation bits, 8 value bits.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LedColor {
    pub hue: u16,
    pub saturation: u8,
    pub value: u8,
}

impl LedColor {
    fn validate(&self) -> Result<(), CrsfParsingError> {
        if self.hue > 359 || self.saturation > 100 || self.value > 100 {
            return Err(CrsfParsingError::InvalidPayload);
        }
        Ok(())
    }

    fn write_to(&self, out: &mut Writer<'_>) -> Result<(), CrsfParsingError> {
        self.validate()?;
        let hs = (self.hue << 7) | u16::from(self.saturation);
        out.bytes(&hs.to_be_bytes())?;
        out.byte(self.value)
    }

    fn read_from(input: &mut Reader<'_>) -> Result<Self, CrsfParsingError> {
        let hs = input.u16()?;
        let color = Self {
            hue: hs >> 7,
            saturation: (hs & 0x7F) as u8,
            value: input.byte()?,
        };
        color.validate()?;
        Ok(color)
    }
}

/// LED commands (command ID 0x09). Pulse durations are in milliseconds.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LedCommand {
    SetDefault,
    OverrideColor(LedColor),
    OverridePulse {
        duration_ms: u16,
        start: LedColor,
        stop: LedColor,
    },
    OverrideBlink {
        interval_ms: u16,
        start: LedColor,
        stop: LedColor,
    },
    OverrideShift {
        interval_ms: u16,
        color: LedColor,
    },
}

impl LedCommand {
    pub(super) fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let mut out = Writer::new(buffer);
        match self {
            Self::SetDefault => out.byte(SUB_COMMAND_ID_LED_SET_DEFAULT)?,
            Self::OverrideColor(color) => {
                out.byte(SUB_COMMAND_ID_LED_OVERRIDE_COLOR)?;
                color.write_to(&mut out)?;
            }
            Self::OverridePulse {
                duration_ms,
                start,
                stop,
            } => {
                out.byte(SUB_COMMAND_ID_LED_OVERRIDE_PULSE)?;
                out.bytes(&duration_ms.to_be_bytes())?;
                start.write_to(&mut out)?;
                stop.write_to(&mut out)?;
            }
            Self::OverrideBlink {
                interval_ms,
                start,
                stop,
            } => {
                out.byte(SUB_COMMAND_ID_LED_OVERRIDE_BLINK)?;
                out.bytes(&interval_ms.to_be_bytes())?;
                start.write_to(&mut out)?;
                stop.write_to(&mut out)?;
            }
            Self::OverrideShift { interval_ms, color } => {
                out.byte(SUB_COMMAND_ID_LED_OVERRIDE_SHIFT)?;
                out.bytes(&interval_ms.to_be_bytes())?;
                color.write_to(&mut out)?;
            }
        }
        Ok(out.position)
    }
}

impl TryFrom<&[u8]> for LedCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut input = Reader::new(data);
        match input.byte()? {
            SUB_COMMAND_ID_LED_SET_DEFAULT => Ok(Self::SetDefault),
            SUB_COMMAND_ID_LED_OVERRIDE_COLOR => {
                Ok(Self::OverrideColor(LedColor::read_from(&mut input)?))
            }
            SUB_COMMAND_ID_LED_OVERRIDE_PULSE => Ok(Self::OverridePulse {
                duration_ms: input.u16()?,
                start: LedColor::read_from(&mut input)?,
                stop: LedColor::read_from(&mut input)?,
            }),
            SUB_COMMAND_ID_LED_OVERRIDE_BLINK => Ok(Self::OverrideBlink {
                interval_ms: input.u16()?,
                start: LedColor::read_from(&mut input)?,
                stop: LedColor::read_from(&mut input)?,
            }),
            SUB_COMMAND_ID_LED_OVERRIDE_SHIFT => Ok(Self::OverrideShift {
                interval_ms: input.u16()?,
                color: LedColor::read_from(&mut input)?,
            }),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}
