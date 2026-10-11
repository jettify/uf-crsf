use super::codec::{Reader, Writer};
use crate::CrsfParsingError;

const SUB_COMMAND_ID_GENERAL_SPEED_PROPOSAL: u8 = 0x70;
const SUB_COMMAND_ID_GENERAL_SPEED_PROPOSAL_RESPONSE: u8 = 0x71;

/// Protocol speed negotiation (command ID 0x0A).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GeneralCommand {
    SpeedProposal {
        port_id: u8,
        /// Proposed UART speed in baud, encoded as a big-endian u32.
        proposed_baudrate: u32,
    },
    SpeedProposalResponse {
        port_id: u8,
        accepted: bool,
    },
}

impl GeneralCommand {
    pub(super) fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let mut out = Writer::new(buffer);
        match self {
            Self::SpeedProposal {
                port_id,
                proposed_baudrate,
            } => {
                out.byte(SUB_COMMAND_ID_GENERAL_SPEED_PROPOSAL)?;
                out.byte(*port_id)?;
                out.bytes(&proposed_baudrate.to_be_bytes())?;
            }
            Self::SpeedProposalResponse { port_id, accepted } => {
                out.byte(SUB_COMMAND_ID_GENERAL_SPEED_PROPOSAL_RESPONSE)?;
                out.byte(*port_id)?;
                out.byte(*accepted as u8)?;
            }
        }
        Ok(out.position)
    }
}

impl TryFrom<&[u8]> for GeneralCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut input = Reader::new(data);
        match input.byte()? {
            SUB_COMMAND_ID_GENERAL_SPEED_PROPOSAL => Ok(Self::SpeedProposal {
                port_id: input.byte()?,
                proposed_baudrate: input.u32()?,
            }),
            SUB_COMMAND_ID_GENERAL_SPEED_PROPOSAL_RESPONSE => Ok(Self::SpeedProposalResponse {
                port_id: input.byte()?,
                accepted: input.boolean()?,
            }),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}
