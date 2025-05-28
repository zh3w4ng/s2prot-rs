// #[derive(Debug, PartialEq)]
// pub struct Replay {
//     pub header: Header,
//     pub details: Details,
//     pub init_data: InitData,
//     pub game_events: Vec<GameEvent>,
//     pub message_events: Vec<MessageEvent>,
//     pub tracker_events: Vec<TrackerEvent>,
// }
//
#[derive(Debug, PartialEq)]
pub struct Version {
    pub flags: u8,
    pub major: u8,
    pub minor: u8,
    pub revision: u8,
    pub build: u32,
    pub base_build: u32,
}

#[derive(Debug, PartialEq)]
pub struct UserData {
    pub signature: Option<Vec<u8>>,
    pub version: Version,
}

#[derive(Debug, PartialEq)]
pub struct ParsedField {
    pub name: String,
    pub value: Option<ParsedFieldType>,
}

#[derive(Debug, PartialEq)]
pub enum ParsedFieldType {
    Bool(bool),
    Blob(String),
    Int(isize),
    FourCC(Vec<u8>),
    Array(Vec<ParsedFieldType>),
    Struct(Vec<ParsedField>),
    BitArray { length: usize, data: Vec<u8> },
}
