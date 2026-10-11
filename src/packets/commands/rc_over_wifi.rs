use super::codec::Reader;
use super::codec::Writer;
use crate::CrsfParsingError;

const SUB_COMMAND_ID_RC_OVER_WIFI_ENABLE: u8 = 0x01;
const SUB_COMMAND_ID_RC_OVER_WIFI_DISABLE: u8 = 0x02;

/// Route RC frames over WiFi (command ID 0x13, game mode).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RcOverWifiCommand {
    Enable,
    Disable,
}

impl RcOverWifiCommand {
    pub(super) fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let mut out = Writer::new(buffer);
        out.byte(match self {
            Self::Enable => SUB_COMMAND_ID_RC_OVER_WIFI_ENABLE,
            Self::Disable => SUB_COMMAND_ID_RC_OVER_WIFI_DISABLE,
        })?;
        Ok(out.position)
    }
}

impl TryFrom<&[u8]> for RcOverWifiCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        match Reader::new(data).byte()? {
            SUB_COMMAND_ID_RC_OVER_WIFI_ENABLE => Ok(Self::Enable),
            SUB_COMMAND_ID_RC_OVER_WIFI_DISABLE => Ok(Self::Disable),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}
