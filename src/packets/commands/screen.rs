use super::codec::{Reader, Writer};
use crate::CrsfParsingError;
use heapless::Vec;

const SUB_COMMAND_ID_SCREEN_POPUP_MESSAGE_START: u8 = 0x01;
const SUB_COMMAND_ID_SCREEN_SELECTION_RETURN_VALUE: u8 = 0x02;

// Full frame overhead: sync, length, type, dst, src, command ID,
// screen subcommand ID, command CRC, and frame CRC.
const MAX_POPUP_DATA_LEN: usize = crate::constants::CRSF_MAX_PACKET_SIZE - 9;
// Required popup fields: two string terminators, timeout, and close button flag.
const MAX_COMMAND_STRING_LEN: usize = MAX_POPUP_DATA_LEN - 4;

/// Screen string bytes, stored without a null terminator. UTF-8 is not required.
///
/// At most 51 bytes can fit in a screen string within a 64-byte frame.
/// The combined pop-up fields may impose a smaller limit when serialized.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandString(Vec<u8, MAX_COMMAND_STRING_LEN>);

impl CommandString {
    /// Rejects embedded null bytes and strings longer than 51 bytes.
    pub fn new(bytes: &[u8]) -> Result<Self, CrsfParsingError> {
        if bytes.contains(&0) {
            return Err(CrsfParsingError::InvalidPayload);
        }
        let mut string = Vec::new();
        string
            .extend_from_slice(bytes)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self(string))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CommandString {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{=[u8]}", self.as_bytes());
    }
}

/// Optional pop-up selection controls. A nonempty label is required on the wire.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScreenSelection {
    pub selection_text: CommandString,
    pub value: u8,
    pub min_value: u8,
    pub max_value: u8,
    pub default_value: u8,
    pub unit: CommandString,
}

/// Additional-data field: an empty field occupies one null byte, while a
/// selection includes its label, values, and unit.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScreenAdditionalData {
    Empty,
    Selection(ScreenSelection),
}

/// Typed fields used to construct or inspect an LCD pop-up.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScreenPopupFields {
    pub header: CommandString,
    pub info: CommandString,
    /// Maximum timeout in seconds.
    pub max_timeout_interval: u8,
    pub close_button_option: bool,
    /// `None` omits the field entirely; `Some(Empty)` writes a null placeholder.
    pub additional_data: Option<ScreenAdditionalData>,
    /// Semicolon-separated response labels, without their null terminator.
    /// Requires an additional-data field (use `Some(Empty)` for no selection).
    pub possible_values: Option<CommandString>,
}

/// Validated pop-up stored in at most 55 inline wire bytes.
///
/// Use `ScreenPopupFields` to construct or inspect its typed fields. Compact
/// storage keeps `Packet` small without requiring a heap allocator.
///
/// ```
/// use uf_crsf::packets::{
///     CommandPayload, CommandString, DirectCommands, ScreenAdditionalData,
///     ScreenCommand, ScreenPopup, ScreenPopupFields,
/// };
///
/// let fields = ScreenPopupFields {
///     header: CommandString::new(b"Confirm")?,
///     info: CommandString::new(b"Reset settings?")?,
///     max_timeout_interval: 10,
///     close_button_option: true,
///     additional_data: Some(ScreenAdditionalData::Empty),
///     possible_values: Some(CommandString::new(b"Yes;No")?),
/// };
/// let command = DirectCommands {
///     dst_addr: 0xEA,
///     src_addr: 0xEE,
///     payload: CommandPayload::Screen(ScreenCommand::PopupMessageStart(
///         ScreenPopup::new(&fields)?,
///     )),
/// };
/// # Ok::<(), uf_crsf::CrsfParsingError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenPopup(Vec<u8, MAX_POPUP_DATA_LEN>);

impl ScreenPopup {
    /// Validates field combinations and the combined 64-byte frame limit.
    pub fn new(fields: &ScreenPopupFields) -> Result<Self, CrsfParsingError> {
        let mut buffer = [0; MAX_POPUP_DATA_LEN];
        let mut out = Writer::new(&mut buffer);
        fields.write_to(&mut out).map_err(|error| match error {
            CrsfParsingError::BufferOverflow => CrsfParsingError::InvalidPayloadLength,
            other => other,
        })?;
        let len = out.position;
        let mut bytes = Vec::new();
        bytes
            .extend_from_slice(&buffer[..len])
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self(bytes))
    }

    /// Decodes the stored bytes into owned, editable fields.
    pub fn fields(&self) -> Result<ScreenPopupFields, CrsfParsingError> {
        ScreenPopupFields::read_from(&mut Reader::new(&self.0))
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ScreenPopup {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ScreenPopup {=[u8]}", &self.0[..]);
    }
}

impl ScreenPopupFields {
    fn write_to(&self, out: &mut Writer<'_>) -> Result<(), CrsfParsingError> {
        if self.possible_values.is_some() && self.additional_data.is_none() {
            return Err(CrsfParsingError::InvalidPayload);
        }
        out.string(&self.header)?;
        out.string(&self.info)?;
        out.byte(self.max_timeout_interval)?;
        out.byte(self.close_button_option as u8)?;
        match &self.additional_data {
            None => {}
            Some(ScreenAdditionalData::Empty) => out.byte(0)?,
            Some(ScreenAdditionalData::Selection(selection)) => {
                if selection.selection_text.as_bytes().is_empty() {
                    return Err(CrsfParsingError::InvalidPayload);
                }
                out.string(&selection.selection_text)?;
                out.bytes(&[
                    selection.value,
                    selection.min_value,
                    selection.max_value,
                    selection.default_value,
                ])?;
                out.string(&selection.unit)?;
            }
        }
        if let Some(values) = &self.possible_values {
            out.string(values)?;
        }
        Ok(())
    }

    fn read_from(input: &mut Reader<'_>) -> Result<Self, CrsfParsingError> {
        let header = input.string()?;
        let info = input.string()?;
        let max_timeout_interval = input.byte()?;
        let close_button_option = input.boolean()?;
        let additional_data = if input.remaining.is_empty() {
            None
        } else {
            let selection_text = input.string()?;
            Some(if selection_text.as_bytes().is_empty() {
                ScreenAdditionalData::Empty
            } else {
                ScreenAdditionalData::Selection(ScreenSelection {
                    selection_text,
                    value: input.byte()?,
                    min_value: input.byte()?,
                    max_value: input.byte()?,
                    default_value: input.byte()?,
                    unit: input.string()?,
                })
            })
        };
        let possible_values = if input.remaining.is_empty() {
            None
        } else {
            Some(input.string()?)
        };
        Ok(Self {
            header,
            info,
            max_timeout_interval,
            close_button_option,
            additional_data,
            possible_values,
        })
    }
}

/// Screen commands (command ID 0x22).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScreenCommand {
    PopupMessageStart(ScreenPopup),
    SelectionReturnValue {
        value: u8,
        /// True to process, false to cancel.
        response: bool,
    },
}

impl ScreenCommand {
    pub(super) fn write_to(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let mut out = Writer::new(buffer);
        match self {
            Self::PopupMessageStart(popup) => {
                out.byte(SUB_COMMAND_ID_SCREEN_POPUP_MESSAGE_START)?;
                out.bytes(&popup.0)?;
            }
            Self::SelectionReturnValue { value, response } => {
                out.byte(SUB_COMMAND_ID_SCREEN_SELECTION_RETURN_VALUE)?;
                out.byte(*value)?;
                out.byte(*response as u8)?;
            }
        }
        Ok(out.position)
    }
}

impl TryFrom<&[u8]> for ScreenCommand {
    type Error = CrsfParsingError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut input = Reader::new(data);
        match input.byte()? {
            SUB_COMMAND_ID_SCREEN_POPUP_MESSAGE_START => {
                let fields = ScreenPopupFields::read_from(&mut input)?;
                Ok(Self::PopupMessageStart(ScreenPopup::new(&fields)?))
            }
            SUB_COMMAND_ID_SCREEN_SELECTION_RETURN_VALUE => Ok(Self::SelectionReturnValue {
                value: input.byte()?,
                response: input.boolean()?,
            }),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }
}
