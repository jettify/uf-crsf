use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use crc::Crc;
use heapless::Vec;

pub const COMMAND_CRC_ALGO: Crc<u8> = Crc::<u8>::new(&crc::Algorithm {
    width: 8,
    poly: 0xBA,
    init: 0x00,
    refin: false,
    refout: false,
    xorout: 0x00,
    check: 0x00,
    residue: 0x00,
});

// Command IDs
const COMMAND_ID_FC: u8 = 0x01;
const COMMAND_ID_OSD: u8 = 0x05;
const COMMAND_ID_VTX: u8 = 0x08;
const COMMAND_ID_CROSSFIRE: u8 = 0x10;
const COMMAND_ID_FLOW_CONTROL: u8 = 0x20;
const COMMAND_ID_ACK: u8 = 0xFF;

// FC Sub-command IDs
const SUB_COMMAND_ID_FC_FORCE_DISARM: u8 = 0x01;
const SUB_COMMAND_ID_FC_SCALE_CHANNEL: u8 = 0x02;

// OSD Sub-command IDs
const SUB_COMMAND_ID_OSD_SEND_BUTTONS: u8 = 0x01;

// VTX Sub-command IDs
const SUB_COMMAND_ID_VTX_SET_FREQUENCY: u8 = 0x02;
const SUB_COMMAND_ID_VTX_ENABLE_PIT_MODE_ON_POWER_UP: u8 = 0x04;
const SUB_COMMAND_ID_VTX_POWER_UP_FROM_PIT_MODE: u8 = 0x05;
const SUB_COMMAND_ID_VTX_SET_DYNAMIC_POWER: u8 = 0x06;
const SUB_COMMAND_ID_VTX_SET_POWER: u8 = 0x08;

// Crossfire Sub-command IDs
const SUB_COMMAND_ID_CROSSFIRE_SET_RECEIVER_IN_BIND_MODE: u8 = 0x01;
const SUB_COMMAND_ID_CROSSFIRE_CANCEL_BIND_MODE: u8 = 0x02;
const SUB_COMMAND_ID_CROSSFIRE_SET_BIND_ID: u8 = 0x03;
const SUB_COMMAND_ID_CROSSFIRE_MODEL_SELECTION: u8 = 0x05;
const SUB_COMMAND_ID_CROSSFIRE_CURRENT_MODEL_SELECTION: u8 = 0x06;
const SUB_COMMAND_ID_CROSSFIRE_REPLY_CURRENT_MODEL_SELECTION: u8 = 0x07;

// Flow Control Sub-command IDs
const SUB_COMMAND_ID_FLOW_CONTROL_SUBSCRIBE: u8 = 0x01;
const SUB_COMMAND_ID_FLOW_CONTROL_UNSUBSCRIBE: u8 = 0x02;

/// Represents a Direct Commands packet (frame type 0x32).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DirectCommands {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub payload: CommandPayload,
}

/// Enum for the different payloads of a DirectCommands packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandPayload {
    Fc(FcCommand),
    Osd(OsdCommand),
    Vtx(VtxCommand),
    Crossfire(CrossfireCommand),
    FlowControl(FlowControlCommand),
    Ack(CommandAck),
}

/// FC Commands (command ID 0x01)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FcCommand {
    ForceDisarm,
    ScaleChannel,
}

/// OSD Commands (command ID 0x05)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OsdCommand {
    SendButtons(u8),
}

/// VTX Commands (command ID 0x08)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VtxCommand {
    SetFrequency(u16),
    EnablePitModeOnPowerUp {
        pit_mode: bool,
        pit_mode_control: u8,
        pit_mode_switch: u8,
    },
    PowerUpFromPitMode,
    SetDynamicPower(u8),
    SetPower(u8),
}

/// Crossfire Commands (command ID 0x10)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CrossfireCommand {
    SetReceiverInBindMode,
    CancelBindMode,
    SetBindId,
    ModelSelection(u8),
    CurrentModelSelection,
    ReplyCurrentModelSelection(u8),
}

/// Flow Control Commands (command ID 0x20)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControlCommand {
    Subscribe {
        frame_type: u8,
        max_interval_time: u16,
    },
    Unsubscribe {
        frame_type: u8,
    },
}

/// Command ACK (command ID 0xFF)
#[derive(Clone, Debug, PartialEq)]
pub struct CommandAck {
    pub command_id: u8,
    pub sub_command_id: u8,
    pub action: u8,               // 0 = rejected, 1 = accepted
    pub information: Vec<u8, 48>, // Variable length string
}

#[cfg(feature = "defmt")]
impl defmt::Format for CommandAck {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "CommandAck {{ command_id: {}, sub_command_id: {}, action: {}, information: {} }}",
            self.command_id,
            self.sub_command_id,
            self.action,
            self.information.as_slice(),
        )
    }
}

impl CrsfPacket for DirectCommands {
    const PACKET_TYPE: PacketType = PacketType::Command;
    // dst, src, cmd_id, crc
    const MIN_PAYLOAD_SIZE: usize = 4;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        // data = [dst, src, cmd_id, payload..., crc]
        let crc_byte_index = data.len() - 1;
        let received_crc = data[crc_byte_index];
        let payload_with_headers = &data[..crc_byte_index];

        // CRC is calculated over [type, dst, src, cmd_id, payload...]
        let mut digest = COMMAND_CRC_ALGO.digest();
        digest.update(&[Self::PACKET_TYPE as u8]);
        digest.update(payload_with_headers);
        let calculated_crc = digest.finalize();

        if received_crc != calculated_crc {
            return Err(CrsfParsingError::InvalidPayload);
        }

        let dst_addr = data[0];
        let src_addr = data[1];
        let command_id = data[2];
        let command_payload_data = &data[3..crc_byte_index];

        let payload = match command_id {
            COMMAND_ID_FC => CommandPayload::Fc(FcCommand::try_from(command_payload_data)?),
            COMMAND_ID_OSD => CommandPayload::Osd(OsdCommand::try_from(command_payload_data)?),
            COMMAND_ID_VTX => CommandPayload::Vtx(VtxCommand::try_from(command_payload_data)?),
            COMMAND_ID_CROSSFIRE => {
                CommandPayload::Crossfire(CrossfireCommand::try_from(command_payload_data)?)
            }
            COMMAND_ID_FLOW_CONTROL => {
                CommandPayload::FlowControl(FlowControlCommand::try_from(command_payload_data)?)
            }
            COMMAND_ID_ACK => CommandPayload::Ack(CommandAck::try_from(command_payload_data)?),
            _ => return Err(CrsfParsingError::InvalidPayload), // Unknown command
        };

        Ok(Self {
            dst_addr,
            src_addr,
            payload,
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        buffer[0] = self.dst_addr;
        buffer[1] = self.src_addr;
        buffer[2] = self.payload.command_id();

        let payload_len = self.payload.write_to(&mut buffer[3..])?;
        let total_len = 3 + payload_len;

        // Calculate and append CRC
        // CRC is over [type, dst, src, cmd_id, payload...]
        let mut digest = COMMAND_CRC_ALGO.digest();
        digest.update(&[Self::PACKET_TYPE as u8]);
        digest.update(&buffer[..total_len]);
        let crc = digest.finalize();

        if buffer.len() < total_len + 1 {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[total_len] = crc;
        Ok(total_len + 1)
    }
}

impl CommandPayload {
    fn command_id(&self) -> u8 {
        match self {
            CommandPayload::Fc(_) => COMMAND_ID_FC,
            CommandPayload::Osd(_) => COMMAND_ID_OSD,
            CommandPayload::Vtx(_) => COMMAND_ID_VTX,
            CommandPayload::Crossfire(_) => COMMAND_ID_CROSSFIRE,
            CommandPayload::FlowControl(_) => COMMAND_ID_FLOW_CONTROL,
            CommandPayload::Ack(_) => COMMAND_ID_ACK,
        }
    }

    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match self {
            CommandPayload::Fc(cmd) => cmd.write_to(buffer),
            CommandPayload::Osd(cmd) => cmd.write_to(buffer),
            CommandPayload::Vtx(cmd) => cmd.write_to(buffer),
            CommandPayload::Crossfire(cmd) => cmd.write_to(buffer),
            CommandPayload::FlowControl(cmd) => cmd.write_to(buffer),
            CommandPayload::Ack(cmd) => cmd.write_to(buffer),
        }
    }
}

impl FcCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let (sub_command_id, len) = match self {
            FcCommand::ForceDisarm => (SUB_COMMAND_ID_FC_FORCE_DISARM, 1),
            FcCommand::ScaleChannel => (SUB_COMMAND_ID_FC_SCALE_CHANNEL, 1),
        };
        buffer[0] = sub_command_id;
        Ok(len)
    }
}

impl<'a> TryFrom<&'a [u8]> for FcCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let sub_command_id = data[0];
        match sub_command_id {
            SUB_COMMAND_ID_FC_FORCE_DISARM => Ok(FcCommand::ForceDisarm),
            SUB_COMMAND_ID_FC_SCALE_CHANNEL => Ok(FcCommand::ScaleChannel),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl OsdCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match self {
            OsdCommand::SendButtons(buttons) => {
                buffer[0] = SUB_COMMAND_ID_OSD_SEND_BUTTONS;
                buffer[1] = *buttons;
                Ok(2)
            }
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for OsdCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let sub_command_id = data[0];
        match sub_command_id {
            SUB_COMMAND_ID_OSD_SEND_BUTTONS => Ok(OsdCommand::SendButtons(data[1])),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl VtxCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match self {
            VtxCommand::SetFrequency(freq) => {
                buffer[0] = SUB_COMMAND_ID_VTX_SET_FREQUENCY;
                buffer[1..3].copy_from_slice(&freq.to_be_bytes());
                Ok(3)
            }
            VtxCommand::EnablePitModeOnPowerUp {
                pit_mode,
                pit_mode_control,
                pit_mode_switch,
            } => {
                buffer[0] = SUB_COMMAND_ID_VTX_ENABLE_PIT_MODE_ON_POWER_UP;
                buffer[1] = (*pit_mode as u8) | (pit_mode_control << 1) | (pit_mode_switch << 3);
                Ok(2)
            }
            VtxCommand::PowerUpFromPitMode => {
                buffer[0] = SUB_COMMAND_ID_VTX_POWER_UP_FROM_PIT_MODE;
                Ok(1)
            }
            VtxCommand::SetDynamicPower(power) => {
                buffer[0] = SUB_COMMAND_ID_VTX_SET_DYNAMIC_POWER;
                buffer[1] = *power;
                Ok(2)
            }
            VtxCommand::SetPower(power) => {
                buffer[0] = SUB_COMMAND_ID_VTX_SET_POWER;
                buffer[1] = *power;
                Ok(2)
            }
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for VtxCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let sub_command_id = data[0];
        let payload = &data[1..];
        match sub_command_id {
            SUB_COMMAND_ID_VTX_SET_FREQUENCY => {
                let freq_bytes: [u8; 2] = payload[0..2]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
                Ok(VtxCommand::SetFrequency(u16::from_be_bytes(freq_bytes)))
            }
            SUB_COMMAND_ID_VTX_ENABLE_PIT_MODE_ON_POWER_UP => {
                let byte = payload[0];
                Ok(VtxCommand::EnablePitModeOnPowerUp {
                    pit_mode: (byte & 0b1) != 0,
                    pit_mode_control: (byte >> 1) & 0b11,
                    pit_mode_switch: (byte >> 3) & 0b1111,
                })
            }
            SUB_COMMAND_ID_VTX_POWER_UP_FROM_PIT_MODE => Ok(VtxCommand::PowerUpFromPitMode),
            SUB_COMMAND_ID_VTX_SET_DYNAMIC_POWER => Ok(VtxCommand::SetDynamicPower(payload[0])),
            SUB_COMMAND_ID_VTX_SET_POWER => Ok(VtxCommand::SetPower(payload[0])),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl CrossfireCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match self {
            CrossfireCommand::SetReceiverInBindMode => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_SET_RECEIVER_IN_BIND_MODE;
                Ok(1)
            }
            CrossfireCommand::CancelBindMode => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_CANCEL_BIND_MODE;
                Ok(1)
            }
            CrossfireCommand::SetBindId => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_SET_BIND_ID;
                Ok(1)
            }
            CrossfireCommand::ModelSelection(model) => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_MODEL_SELECTION;
                buffer[1] = *model;
                Ok(2)
            }
            CrossfireCommand::CurrentModelSelection => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_CURRENT_MODEL_SELECTION;
                Ok(1)
            }
            CrossfireCommand::ReplyCurrentModelSelection(model) => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_REPLY_CURRENT_MODEL_SELECTION;
                buffer[1] = *model;
                Ok(2)
            }
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for CrossfireCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let sub_command_id = data[0];
        let payload = &data[1..];
        match sub_command_id {
            SUB_COMMAND_ID_CROSSFIRE_SET_RECEIVER_IN_BIND_MODE => {
                Ok(CrossfireCommand::SetReceiverInBindMode)
            }
            SUB_COMMAND_ID_CROSSFIRE_CANCEL_BIND_MODE => Ok(CrossfireCommand::CancelBindMode),
            SUB_COMMAND_ID_CROSSFIRE_SET_BIND_ID => Ok(CrossfireCommand::SetBindId),
            SUB_COMMAND_ID_CROSSFIRE_MODEL_SELECTION => {
                Ok(CrossfireCommand::ModelSelection(payload[0]))
            }
            SUB_COMMAND_ID_CROSSFIRE_CURRENT_MODEL_SELECTION => {
                Ok(CrossfireCommand::CurrentModelSelection)
            }
            SUB_COMMAND_ID_CROSSFIRE_REPLY_CURRENT_MODEL_SELECTION => {
                Ok(CrossfireCommand::ReplyCurrentModelSelection(payload[0]))
            }
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl FlowControlCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match self {
            FlowControlCommand::Subscribe {
                frame_type,
                max_interval_time,
            } => {
                buffer[0] = SUB_COMMAND_ID_FLOW_CONTROL_SUBSCRIBE;
                buffer[1] = *frame_type;
                buffer[2..4].copy_from_slice(&max_interval_time.to_be_bytes());
                Ok(4)
            }
            FlowControlCommand::Unsubscribe { frame_type } => {
                buffer[0] = SUB_COMMAND_ID_FLOW_CONTROL_UNSUBSCRIBE;
                buffer[1] = *frame_type;
                Ok(2)
            }
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for FlowControlCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let sub_command_id = data[0];
        let payload = &data[1..];
        match sub_command_id {
            SUB_COMMAND_ID_FLOW_CONTROL_SUBSCRIBE => {
                let max_interval_time_bytes: [u8; 2] = payload[1..3]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
                Ok(FlowControlCommand::Subscribe {
                    frame_type: payload[0],
                    max_interval_time: u16::from_be_bytes(max_interval_time_bytes),
                })
            }
            SUB_COMMAND_ID_FLOW_CONTROL_UNSUBSCRIBE => Ok(FlowControlCommand::Unsubscribe {
                frame_type: payload[0],
            }),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl CommandAck {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let required_len = 3 + self.information.len();
        if buffer.len() < required_len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0] = self.command_id;
        buffer[1] = self.sub_command_id;
        buffer[2] = self.action;
        buffer[3..required_len].copy_from_slice(&self.information);
        Ok(required_len)
    }
}

impl<'a> TryFrom<&'a [u8]> for CommandAck {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 3 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(CommandAck {
            command_id: data[0],
            sub_command_id: data[1],
            action: data[2],
            information: Vec::from_slice(&data[3..])
                .map_err(|_| CrsfParsingError::BufferOverflow)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_round_trip(packet: &DirectCommands) {
        let mut buffer = [0u8; 64];
        let len = packet.to_bytes(&mut buffer).unwrap();
        let round_trip = DirectCommands::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(packet, &round_trip);
    }

    #[test]
    fn test_fc_command_force_disarm() {
        test_round_trip(&DirectCommands {
            dst_addr: 0xC8,
            src_addr: 0xEA,
            payload: CommandPayload::Fc(FcCommand::ForceDisarm),
        });
    }

    #[test]
    fn test_osd_send_buttons() {
        test_round_trip(&DirectCommands {
            dst_addr: 0x80,
            src_addr: 0xEA,
            payload: CommandPayload::Osd(OsdCommand::SendButtons(0b10101000)),
        });
    }

    #[test]
    fn test_vtx_set_frequency() {
        test_round_trip(&DirectCommands {
            dst_addr: 0xCE,
            src_addr: 0xEA,
            payload: CommandPayload::Vtx(VtxCommand::SetFrequency(5800)),
        });
    }

    #[test]
    fn test_crossfire_model_selection() {
        test_round_trip(&DirectCommands {
            dst_addr: 0xEE,
            src_addr: 0xEA,
            payload: CommandPayload::Crossfire(CrossfireCommand::ModelSelection(5)),
        });
    }

    #[test]
    fn test_flow_control_subscribe() {
        test_round_trip(&DirectCommands {
            dst_addr: 0xC8,
            src_addr: 0xEA,
            payload: CommandPayload::FlowControl(FlowControlCommand::Subscribe {
                frame_type: 0x14, // Link Statistics
                max_interval_time: 1000,
            }),
        });
    }

    #[test]
    fn test_command_ack() {
        test_round_trip(&DirectCommands {
            dst_addr: 0xEA,
            src_addr: 0xEE,
            payload: CommandPayload::Ack(CommandAck {
                command_id: 0x10,
                sub_command_id: 0x01,
                action: 1,
                information: Vec::from_slice(b"OK").unwrap(),
            }),
        });
    }

    #[test]
    fn test_from_bytes_invalid_crc() {
        let data: [u8; 5] = [0xC8, 0xEA, 0x01, 0x01, 0x00]; // wrong crc
        let result = DirectCommands::from_bytes(&data);
        assert!(matches!(result, Err(CrsfParsingError::InvalidPayload)));
    }
}
