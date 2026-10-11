use super::codec::Reader;
use super::codec::Writer;
use crate::CrsfParsingError;

const SUB_COMMAND_ID_BLUETOOTH_RESET: u8 = 0x01;
const SUB_COMMAND_ID_BLUETOOTH_ENABLE: u8 = 0x02;
const SUB_COMMAND_ID_BLUETOOTH_ECHO: u8 = 0x64;

/// Bluetooth commands (command ID 0x03).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BluetoothCommand {
    Reset,
    Enable(bool),
    /// Bare echo command; the specification defines no additional payload.
    Echo,
}

impl BluetoothCommand {
    pub(super) fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let mut out = Writer::new(buffer);
        match self {
            Self::Reset => out.byte(SUB_COMMAND_ID_BLUETOOTH_RESET)?,
            Self::Enable(enabled) => {
                out.byte(SUB_COMMAND_ID_BLUETOOTH_ENABLE)?;
                out.byte(*enabled as u8)?;
            }
            Self::Echo => out.byte(SUB_COMMAND_ID_BLUETOOTH_ECHO)?,
        }
        Ok(out.position)
    }
}

impl TryFrom<&[u8]> for BluetoothCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut input = Reader::new(data);
        match input.byte()? {
            SUB_COMMAND_ID_BLUETOOTH_RESET => Ok(Self::Reset),
            SUB_COMMAND_ID_BLUETOOTH_ENABLE => Ok(Self::Enable(input.boolean()?)),
            SUB_COMMAND_ID_BLUETOOTH_ECHO => Ok(Self::Echo),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}
