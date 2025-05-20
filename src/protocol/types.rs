use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_index: usize,
    pub tag: isize,
}

#[derive(Debug, PartialEq)]
pub struct EventType {
    pub event_name: String,
    pub event_id: u16,
    pub type_index: usize,
}

#[derive(Debug, PartialEq)]
pub enum TypeInfo {
    Int {
        offset: isize,
        length: usize,
    },
    Bool,
    Blob {
        offset: isize,
        length: usize,
    },
    BitArray {
        offset: isize,
        length: usize,
    },
    Array {
        offset: isize,
        length: usize,
        type_index: usize,
    },
    Optional {
        type_index: usize,
    },
    Choice {
        offset: isize,
        length: usize,
        fields: Vec<Field>,
    },
    Struct {
        fields: Vec<Field>,
    },
    FourCC,
    Null,
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeInfo::Int { offset, length } => {
                write!(f, "Int(offset: {}, length: {})", offset, length)
            }
            TypeInfo::Bool => write!(f, "Bool"),
            TypeInfo::Blob { offset, length } => {
                write!(f, "Blob(offset: {}, length: {})", offset, length)
            }
            TypeInfo::BitArray { offset, length } => {
                write!(f, "BitArray(offset: {}, length: {})", offset, length)
            }
            TypeInfo::Array {
                offset,
                length,
                type_index,
            } => {
                write!(
                    f,
                    "Array(offset: {}, length: {}, type_index: {})",
                    offset, length, type_index
                )
            }
            TypeInfo::Optional { type_index } => write!(f, "Optional(type_index: {})", type_index),
            TypeInfo::Choice {
                offset,
                length,
                fields,
            } => {
                write!(
                    f,
                    "Choice(offset: {}, length: {}, fields: {:?})",
                    offset, length, fields
                )
            }
            TypeInfo::Struct { fields } => write!(f, "Struct(fields: {:?})", fields),
            TypeInfo::FourCC => write!(f, "FourCC"),
            TypeInfo::Null => write!(f, "Null"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Protocol {
    pub build_version: u32,
    pub has_tracker_events: bool,
    pub type_infos: Vec<TypeInfo>,
    pub game_event_types: HashMap<u16, EventType>,
    pub game_eventid_type_index: Option<usize>,
    pub message_event_types: HashMap<u16, EventType>,
    pub message_eventid_type_index: Option<usize>,
    pub tracker_event_types: HashMap<u16, EventType>,
    pub tracker_eventid_type_index: Option<usize>,
    pub game_loop_type_index: Option<usize>,
    pub replay_userid_type_index: Option<usize>,
    pub replay_header_type_index: Option<usize>,
    pub game_details_type_index: Option<usize>,
    pub replay_initdata_type_index: Option<usize>,
}
