#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_index: u16,
}

#[derive(Debug, PartialEq)]
pub struct EventType {
    pub event_name: String,
    pub event_id: u16,
    pub type_index: u16,
}

#[derive(Debug, PartialEq)]
pub enum TypeInfo {
    Int {
        offset: usize,
        length: usize,
    },
    Bool,
    Blob {
        offset: usize,
        length: usize,
    },
    Array {
        offset: usize,
        length: usize,
        type_index: u16,
    },
    Optional {
        type_index: u16,
    },
    Choice {
        offset: usize,
        length: usize,
        fields: Vec<Field>,
    },
    Struct {
        fields: Vec<Field>,
    },
    FourCC,
}

#[derive(Debug, PartialEq)]
pub struct Protocol {
    pub build_version: u16,
    pub typo_infos: Vec<TypeInfo>,
    pub has_tracker_events: bool,
    pub game_events_types: Vec<EventType>,
    pub game_eventid_type_index: u16,
    pub message_events_types: Vec<EventType>,
    pub message_eventid_type_index: u16,
    pub tracker_events_types: Vec<EventType>,
    pub tracker_eventid_type_index: u16,
    pub game_loop_type_index: usize,
    pub replay_userid_type_index: u16,
    pub replay_header_type_index: u16,
    pub game_details_type_index: u16,
    pub replay_initdata_type_index: u16,
}
