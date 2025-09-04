use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use heapless::String;

const MAX_DEVICE_NAME_LEN: usize = 43;
const EXTENDED_HEADER_SIZE: usize = 2 * size_of::<u8>();
const FIXED_FIELDS_SIZE: usize = 3 * size_of::<u32>() + 2 * size_of::<u8>();

/// Represents a Device Information packet (0x29).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInformation {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub device_name: String<MAX_DEVICE_NAME_LEN>,
    pub serial_number: u32,
    pub hardware_id: u32,
    pub firmware_id: u32,
    pub parameters_total: u8,
    pub parameter_version_number: u8,
}

#[cfg(feature = "defmt")]
impl defmt::Format for DeviceInformation {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "DeviceInformation {{ dst_addr: {=u8}, src_addrs: {=u8}, device_name: {}, serial_number: {=u32}, hardware_id: {=u32}, firmware_id: {=u32}, parameters_total: {=u8}, parameter_version_number: {=u8} }}",
            self.dst_addr,
            self.src_addr,
            self.device_name.as_str(),
            self.serial_number,
            self.hardware_id,
            self.firmware_id,
            self.parameters_total,
            self.parameter_version_number,
        )
    }
}

impl CrsfPacket for DeviceInformation {
    const PACKET_TYPE: PacketType = PacketType::DeviceInfo;
    // Minimum payload is dst, src, a null terminator for the string + 14 bytes of other data
    const MIN_PAYLOAD_SIZE: usize = EXTENDED_HEADER_SIZE + 1 + FIXED_FIELDS_SIZE;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let name_bytes = self.device_name.as_bytes();
        let name_len = name_bytes.len();
        let payload_len = EXTENDED_HEADER_SIZE + name_len + 1 + FIXED_FIELDS_SIZE;

        if buffer.len() < payload_len {
            return Err(CrsfParsingError::BufferOverflow);
        }

        buffer[0] = self.dst_addr;
        buffer[1] = self.src_addr;

        let mut offset = EXTENDED_HEADER_SIZE;
        buffer[offset..offset + name_len].copy_from_slice(name_bytes);
        offset += name_len;
        buffer[offset] = 0; // Null terminator
        offset += 1;

        buffer[offset..offset + 4].copy_from_slice(&self.serial_number.to_be_bytes());
        offset += 4;
        buffer[offset..offset + 4].copy_from_slice(&self.hardware_id.to_be_bytes());
        offset += 4;
        buffer[offset..offset + 4].copy_from_slice(&self.firmware_id.to_be_bytes());
        offset += 4;
        buffer[offset] = self.parameters_total;
        offset += 1;
        buffer[offset] = self.parameter_version_number;

        Ok(payload_len)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let dst_addr = data[0];
        let src_addr = data[1];

        let payload = &data[EXTENDED_HEADER_SIZE..];
        let null_pos = payload
            .iter()
            .position(|&b| b == 0)
            .ok_or(CrsfParsingError::InvalidPayload)?;
        let s = core::str::from_utf8(&payload[..null_pos])
            .map_err(|_| CrsfParsingError::InvalidPayload)?;
        let mut device_name = String::new();
        device_name
            .push_str(s)
            .map_err(|_e| CrsfParsingError::InvalidPayloadLength)?;

        let mut offset = null_pos + 1;
        if payload.len() < offset + FIXED_FIELDS_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let serial_number = u32::from_be_bytes(payload[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let hardware_id = u32::from_be_bytes(payload[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let firmware_id = u32::from_be_bytes(payload[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let parameters_total = payload[offset];
        offset += 1;
        let parameter_version_number = payload[offset];

        Ok(Self {
            dst_addr,
            src_addr,
            device_name,
            serial_number,
            hardware_id,
            firmware_id,
            parameters_total,
            parameter_version_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_information_to_bytes() {
        let mut device_name = String::new();
        device_name.push_str("TBS Tracer").unwrap();
        let info = DeviceInformation {
            dst_addr: 0xEA,
            src_addr: 0xEE,
            device_name,
            serial_number: 0x12345678,
            hardware_id: 0xABCDEF01,
            firmware_id: 0x98765432,
            parameters_total: 42,
            parameter_version_number: 5,
        };

        let mut buffer = [0u8; 60];
        let len = info.to_bytes(&mut buffer).unwrap();

        let expected_name = b"TBS Tracer\0";
        let expected_len = EXTENDED_HEADER_SIZE + expected_name.len() + FIXED_FIELDS_SIZE;
        assert_eq!(len, expected_len);

        assert_eq!(buffer[0], 0xEA);
        assert_eq!(buffer[1], 0xEE);
        assert_eq!(
            &buffer[EXTENDED_HEADER_SIZE..EXTENDED_HEADER_SIZE + expected_name.len()],
            expected_name
        );
        let mut offset = EXTENDED_HEADER_SIZE + expected_name.len();
        assert_eq!(&buffer[offset..offset + 4], &0x12345678u32.to_be_bytes());
        offset += 4;
        assert_eq!(&buffer[offset..offset + 4], &0xABCDEF01u32.to_be_bytes());
        offset += 4;
        assert_eq!(&buffer[offset..offset + 4], &0x98765432u32.to_be_bytes());
        offset += 4;
        assert_eq!(buffer[offset], 42);
        offset += 1;
        assert_eq!(buffer[offset], 5);
    }

    #[test]
    fn test_device_information_from_bytes() {
        let data =
            b"\xEA\xEE\nTBS Tracer\0\x12\x34\x56\x78\xAB\xCD\xEF\x01\x98\x76\x54\x32\x2A\x05";
        let info = DeviceInformation::from_bytes(data).unwrap();

        let mut expected_name: String<MAX_DEVICE_NAME_LEN> = String::new();
        expected_name.push_str("\nTBS Tracer").unwrap();

        assert_eq!(info.dst_addr, 0xEA);
        assert_eq!(info.src_addr, 0xEE);
        assert_eq!(info.device_name, expected_name);
        assert_eq!(info.serial_number, 0x12345678);
        assert_eq!(info.hardware_id, 0xABCDEF01);
        assert_eq!(info.firmware_id, 0x98765432);
        assert_eq!(info.parameters_total, 42);
        assert_eq!(info.parameter_version_number, 5);
    }

    #[test]
    fn test_device_information_round_trip() {
        let mut device_name = String::new();
        device_name.push_str("MyDevice").unwrap();
        let info = DeviceInformation {
            dst_addr: 0x12,
            src_addr: 0x34,
            device_name,
            serial_number: 1,
            hardware_id: 2,
            firmware_id: 3,
            parameters_total: 4,
            parameter_version_number: 5,
        };

        let mut buffer = [0u8; 60];
        let len = info.to_bytes(&mut buffer).unwrap();
        let round_trip_info = DeviceInformation::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(info, round_trip_info);
    }

    #[test]
    fn test_device_information_from_bytes_invalid_len_too_short() {
        let data = b"\xEA\xEEToo short\0";
        let result = DeviceInformation::from_bytes(data);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));
    }

    #[test]
    fn test_device_information_from_bytes_invalid_len_no_room_for_fixed_fields() {
        let data = b"\xEA\xEEThis string is long enough but no room for fixed fields\0";
        let result = DeviceInformation::from_bytes(data);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));
    }

    #[test]
    fn test_device_information_from_bytes_no_null() {
        let data = b"\xEA\xEE\nNo null terminator here and lots of other data that should be enough for the rest of the fields 12345678901234";
        let result = DeviceInformation::from_bytes(data);
        assert!(matches!(result, Err(CrsfParsingError::InvalidPayload)));
    }
}
