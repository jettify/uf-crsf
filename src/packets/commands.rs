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

// Full frame overhead: sync, length, type, dst, src, command ID,
// three ACK fields, string terminator, command CRC, and frame CRC.
const MAX_ACK_INFORMATION_LEN: usize = crate::constants::CRSF_MAX_PACKET_SIZE - 12;

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
    /// Frequency in MHz, from 5000 through 6000 inclusive.
    SetFrequency(u16),
    EnablePitModeOnPowerUp {
        pit_mode: bool,
        /// 0 = off, 1 = on, 2 = arm, 3 = failsafe.
        pit_mode_control: u8,
        /// 0 = Ch5, 1 = Ch5 inverted, through 15 = Ch12 inverted.
        pit_mode_switch: u8,
    },
    PowerUpFromPitMode,
    /// Power in dBm; 0 can be used for pit-mode power.
    /// Send at 1 Hz. After 3 seconds without an update, the VTX reverts
    /// to the power configured by `SetPower`.
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
    pub action: u8, // 0 = rejected, 1 = accepted
    information: Vec<u8, MAX_ACK_INFORMATION_LEN>,
}

impl CommandAck {
    /// Creates an ACK with up to 52 information bytes, excluding the terminator.
    ///
    /// `information` must not contain null bytes. Serialization appends the
    /// terminator. `action` must be 0 (rejected) or 1 (accepted).
    pub fn new(
        command_id: u8,
        sub_command_id: u8,
        action: u8,
        information: &[u8],
    ) -> Result<Self, CrsfParsingError> {
        if information.len() > MAX_ACK_INFORMATION_LEN {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        if action > 1 || information.contains(&0) {
            return Err(CrsfParsingError::InvalidPayload);
        }
        let mut info = Vec::new();
        info.extend_from_slice(information)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            command_id,
            sub_command_id,
            action,
            information: info,
        })
    }

    /// Returns the information bytes without the null terminator.
    pub fn information(&self) -> &[u8] {
        &self.information
    }
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
            self.information(),
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
        if buffer.len() < 3 {
            return Err(CrsfParsingError::BufferOverflow);
        }
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
        if buffer.is_empty() {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0] = match self {
            FcCommand::ForceDisarm => SUB_COMMAND_ID_FC_FORCE_DISARM,
            FcCommand::ScaleChannel => SUB_COMMAND_ID_FC_SCALE_CHANNEL,
        };
        Ok(1)
    }
}

impl<'a> TryFrom<&'a [u8]> for FcCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
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
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
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
        if data.is_empty() {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let sub_command_id = data[0];
        match sub_command_id {
            SUB_COMMAND_ID_OSD_SEND_BUTTONS => {
                if data.len() < 2 {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                Ok(OsdCommand::SendButtons(data[1]))
            }
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl VtxCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match self {
            VtxCommand::SetFrequency(freq) => {
                if !(5000..=6000).contains(freq) {
                    return Err(CrsfParsingError::InvalidPayload);
                }
                if buffer.len() < 3 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = SUB_COMMAND_ID_VTX_SET_FREQUENCY;
                buffer[1..3].copy_from_slice(&freq.to_be_bytes());
                Ok(3)
            }
            VtxCommand::EnablePitModeOnPowerUp {
                pit_mode,
                pit_mode_control,
                pit_mode_switch,
            } => {
                if *pit_mode_control > 3 || *pit_mode_switch > 15 {
                    return Err(CrsfParsingError::InvalidPayload);
                }
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = SUB_COMMAND_ID_VTX_ENABLE_PIT_MODE_ON_POWER_UP;
                buffer[1] = (*pit_mode as u8) | (pit_mode_control << 1) | (pit_mode_switch << 3);
                Ok(2)
            }
            VtxCommand::PowerUpFromPitMode => {
                if buffer.is_empty() {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = SUB_COMMAND_ID_VTX_POWER_UP_FROM_PIT_MODE;
                Ok(1)
            }
            VtxCommand::SetDynamicPower(power) => {
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = SUB_COMMAND_ID_VTX_SET_DYNAMIC_POWER;
                buffer[1] = *power;
                Ok(2)
            }
            VtxCommand::SetPower(power) => {
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
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
        if data.is_empty() {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let sub_command_id = data[0];
        let payload = &data[1..];
        match sub_command_id {
            SUB_COMMAND_ID_VTX_SET_FREQUENCY => {
                if payload.len() < 2 {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let freq_bytes: [u8; 2] = payload[0..2]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
                let frequency = u16::from_be_bytes(freq_bytes);
                if !(5000..=6000).contains(&frequency) {
                    return Err(CrsfParsingError::InvalidPayload);
                }
                Ok(VtxCommand::SetFrequency(frequency))
            }
            SUB_COMMAND_ID_VTX_ENABLE_PIT_MODE_ON_POWER_UP => {
                if payload.is_empty() {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let byte = payload[0];
                Ok(VtxCommand::EnablePitModeOnPowerUp {
                    pit_mode: (byte & 0b1) != 0,
                    pit_mode_control: (byte >> 1) & 0b11,
                    pit_mode_switch: (byte >> 3) & 0b1111,
                })
            }
            SUB_COMMAND_ID_VTX_POWER_UP_FROM_PIT_MODE => Ok(VtxCommand::PowerUpFromPitMode),
            SUB_COMMAND_ID_VTX_SET_DYNAMIC_POWER => {
                if payload.is_empty() {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                Ok(VtxCommand::SetDynamicPower(payload[0]))
            }
            SUB_COMMAND_ID_VTX_SET_POWER => {
                if payload.is_empty() {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                Ok(VtxCommand::SetPower(payload[0]))
            }
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl CrossfireCommand {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        if buffer.is_empty() {
            return Err(CrsfParsingError::BufferOverflow);
        }
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
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_MODEL_SELECTION;
                buffer[1] = *model;
                Ok(2)
            }
            CrossfireCommand::CurrentModelSelection => {
                buffer[0] = SUB_COMMAND_ID_CROSSFIRE_CURRENT_MODEL_SELECTION;
                Ok(1)
            }
            CrossfireCommand::ReplyCurrentModelSelection(model) => {
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
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
        if data.is_empty() {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let sub_command_id = data[0];
        let payload = &data[1..];
        match sub_command_id {
            SUB_COMMAND_ID_CROSSFIRE_SET_RECEIVER_IN_BIND_MODE => {
                Ok(CrossfireCommand::SetReceiverInBindMode)
            }
            SUB_COMMAND_ID_CROSSFIRE_CANCEL_BIND_MODE => Ok(CrossfireCommand::CancelBindMode),
            SUB_COMMAND_ID_CROSSFIRE_SET_BIND_ID => Ok(CrossfireCommand::SetBindId),
            SUB_COMMAND_ID_CROSSFIRE_MODEL_SELECTION => {
                if payload.is_empty() {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                Ok(CrossfireCommand::ModelSelection(payload[0]))
            }
            SUB_COMMAND_ID_CROSSFIRE_CURRENT_MODEL_SELECTION => {
                Ok(CrossfireCommand::CurrentModelSelection)
            }
            SUB_COMMAND_ID_CROSSFIRE_REPLY_CURRENT_MODEL_SELECTION => {
                if payload.is_empty() {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
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
                if buffer.len() < 4 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = SUB_COMMAND_ID_FLOW_CONTROL_SUBSCRIBE;
                buffer[1] = *frame_type;
                buffer[2..4].copy_from_slice(&max_interval_time.to_be_bytes());
                Ok(4)
            }
            FlowControlCommand::Unsubscribe { frame_type } => {
                if buffer.len() < 2 {
                    return Err(CrsfParsingError::BufferOverflow);
                }
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
        if data.is_empty() {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let sub_command_id = data[0];
        let payload = &data[1..];
        match sub_command_id {
            SUB_COMMAND_ID_FLOW_CONTROL_SUBSCRIBE => {
                if payload.len() < 3 {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let max_interval_time_bytes: [u8; 2] = payload[1..3]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
                Ok(FlowControlCommand::Subscribe {
                    frame_type: payload[0],
                    max_interval_time: u16::from_be_bytes(max_interval_time_bytes),
                })
            }
            SUB_COMMAND_ID_FLOW_CONTROL_UNSUBSCRIBE => {
                if payload.is_empty() {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                Ok(FlowControlCommand::Unsubscribe {
                    frame_type: payload[0],
                })
            }
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}

impl CommandAck {
    fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        if self.action > 1 {
            return Err(CrsfParsingError::InvalidPayload);
        }
        let required_len = 4 + self.information.len();
        if buffer.len() < required_len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0] = self.command_id;
        buffer[1] = self.sub_command_id;
        buffer[2] = self.action;
        buffer[3..required_len - 1].copy_from_slice(&self.information);
        buffer[required_len - 1] = 0;
        Ok(required_len)
    }
}

impl<'a> TryFrom<&'a [u8]> for CommandAck {
    type Error = CrsfParsingError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 3 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let end = data[3..]
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(CrsfParsingError::InvalidPayloadLength)?;
        Self::new(data[0], data[1], data[2], &data[3..3 + end])
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
        // make sure buffer overflow handled as proper error
        let mut small_buffer = [0u8; 3];
        let result = packet.to_bytes(&mut small_buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
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
            payload: CommandPayload::Ack(CommandAck::new(0x10, 0x01, 1, b"OK").unwrap()),
        });
    }

    #[test]
    fn test_from_bytes_invalid_crc() {
        let data: [u8; 5] = [0xC8, 0xEA, 0x01, 0x01, 0x00]; // wrong crc
        let result = DirectCommands::from_bytes(&data);
        assert!(matches!(result, Err(CrsfParsingError::InvalidPayload)));
    }

    #[test]
    fn test_small_buffer() {
        let packet = DirectCommands {
            dst_addr: 0xC8,
            src_addr: 0xEA,
            payload: CommandPayload::Fc(FcCommand::ForceDisarm),
        };

        let mut buffer = [0u8; 2];
        let result = packet.to_bytes(&mut buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
    }

    #[test]
    fn test_command_ack_information_accessor() {
        let ack = CommandAck::new(0x10, 0x01, 1, b"OK").unwrap();
        assert_eq!(ack.information(), b"OK");
    }

    #[test]
    fn test_command_ack_new_too_long_information() {
        let information = [b'A'; 53];
        let result = CommandAck::new(0x10, 0x01, 1, &information);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }

    #[test]
    fn test_ack_wire_fixture_and_buffer_boundaries() {
        // Both CRCs were computed independently with bitwise polynomial division.
        let expected = [
            0xC8, 0x0C, 0x32, 0xEA, 0xEE, 0xFF, 0x10, 1, 1, b'O', b'K', 0, 0xAA, 0xA3,
        ];
        let packet = DirectCommands {
            dst_addr: 0xEA,
            src_addr: 0xEE,
            payload: CommandPayload::Ack(CommandAck::new(0x10, 1, 1, b"OK").unwrap()),
        };
        assert_eq!(
            DirectCommands::from_bytes(&expected[3..13]),
            Ok(packet.clone())
        );
        let mut frame = [0; 64];
        let len = crate::packets::write_packet_to_buffer(
            &mut frame,
            crate::packets::PacketAddress::FlightController,
            &packet,
        )
        .unwrap();
        assert_eq!(&frame[..len], &expected);

        // Include the boundary where the text fits but its terminator or CRC does not.
        for size in 0..=10 {
            let mut buffer = [0; 10];
            let result = packet.to_bytes(&mut buffer[..size]);
            if size < 10 {
                assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
            } else {
                assert_eq!(result, Ok(10));
                assert_eq!(&buffer, &expected[3..13]);
            }
        }
    }

    #[test]
    fn test_ack_empty_and_maximum_information() {
        for information in [&b""[..], &[b'A'; 52][..]] {
            let packet = DirectCommands {
                dst_addr: 0xEA,
                src_addr: 0xEE,
                payload: CommandPayload::Ack(CommandAck::new(0x10, 1, 0, information).unwrap()),
            };
            let mut frame = [0; 64];
            let len = crate::packets::write_packet_to_buffer(
                &mut frame,
                crate::packets::PacketAddress::FlightController,
                &packet,
            )
            .unwrap();
            assert_eq!(len, 12 + information.len());
            assert_eq!(frame[len - 3], 0);
            assert_eq!(DirectCommands::from_bytes(&frame[3..len - 1]), Ok(packet));
        }
    }

    #[test]
    fn test_ack_ignores_extensions_after_terminator() {
        let mut data = [0xA5; 60];
        data[..6].copy_from_slice(&[0x10, 1, 1, b'O', b'K', 0]);
        let ack = CommandAck::try_from(&data[..]).unwrap();
        assert_eq!(ack.information(), b"OK");
        assert_eq!(ack, CommandAck::new(0x10, 1, 1, b"OK").unwrap());
    }

    #[test]
    fn test_ack_rejects_malformed_information() {
        for data in [
            &b""[..],
            &[0x10, 1][..],
            &[0x10, 1, 1][..],
            &[0x10, 1, 1, b'A'][..],
        ] {
            assert_eq!(
                CommandAck::try_from(data),
                Err(CrsfParsingError::InvalidPayloadLength)
            );
        }
        let mut data = [b'A'; 57];
        data[..3].copy_from_slice(&[0x10, 1, 1]);
        data[56] = 0;
        assert_eq!(
            CommandAck::try_from(&data[..]),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
        for information in [&b"\0"[..], &b"OK\0"[..], &b"O\0K"[..]] {
            assert_eq!(
                CommandAck::new(0x10, 1, 1, information),
                Err(CrsfParsingError::InvalidPayload)
            );
        }
    }

    #[test]
    fn test_ack_rejects_invalid_actions() {
        for action in [2, 255] {
            assert_eq!(
                CommandAck::new(0x10, 1, action, b""),
                Err(CrsfParsingError::InvalidPayload)
            );
            assert_eq!(
                CommandAck::try_from(&[0x10, 1, action, 0][..]),
                Err(CrsfParsingError::InvalidPayload)
            );
            // The action field is public, so serialization must validate it too.
            let mut ack = CommandAck::new(0x10, 1, 1, b"").unwrap();
            ack.action = action;
            assert_eq!(
                ack.write_to(&mut [0; 4]),
                Err(CrsfParsingError::InvalidPayload)
            );
        }
    }

    #[test]
    fn test_vtx_frequency_limits() {
        for frequency in [0u16, 4999, 5000, 6000, 6001, u16::MAX] {
            let mut buffer = [0; 3];
            let result = VtxCommand::SetFrequency(frequency).write_to(&mut buffer);
            let [hi, lo] = frequency.to_be_bytes();
            let wire = [2, hi, lo];
            let parsed = VtxCommand::try_from(&wire[..]);
            if (5000..=6000).contains(&frequency) {
                assert_eq!(result, Ok(3));
                assert_eq!(buffer, wire);
                assert_eq!(parsed, Ok(VtxCommand::SetFrequency(frequency)));
            } else {
                assert_eq!(result, Err(CrsfParsingError::InvalidPayload));
                assert_eq!(parsed, Err(CrsfParsingError::InvalidPayload));
            }
        }
    }

    #[test]
    fn test_vtx_pit_mode_field_limits() {
        let command = VtxCommand::EnablePitModeOnPowerUp {
            pit_mode: true,
            pit_mode_control: 3,
            pit_mode_switch: 15,
        };
        let mut buffer = [0; 2];
        assert_eq!(command.write_to(&mut buffer), Ok(2));
        assert_eq!(buffer, [4, 0x7F]);
        assert_eq!(VtxCommand::try_from(&buffer[..]), Ok(command));
        for (control, switch) in [(4, 0), (255, 0), (0, 16), (0, 255)] {
            let command = VtxCommand::EnablePitModeOnPowerUp {
                pit_mode: false,
                pit_mode_control: control,
                pit_mode_switch: switch,
            };
            assert_eq!(
                command.write_to(&mut buffer),
                Err(CrsfParsingError::InvalidPayload)
            );
        }
    }
}
