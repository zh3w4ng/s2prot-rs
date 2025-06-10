pub mod buffer;
pub mod decoders;
pub mod types;

use crate::protocol::types::{EventType, Protocol, TypeInfo};
use buffer::BitPackedBuff;
use decoders::{raw_decode, versioned_decode};
use mpq::Archive;
use serde_json::Value;
use std::collections::HashMap;
use std::str;
use types::*;

pub fn build_replay(file_name: &str, protocol: &Protocol) -> Vec<ParsedField> {
    let mut archive = load_mpq_archive(file_name);
    // let parsed_user_data = decode_user_data(&mut archive, protocol);
    list_files_in_archive(&mut archive);
    // let parsed_details_data = decode_details_data(&mut archive, protocol);
    // let parsed_init_data = decode_init_data(&mut archive, protocol);
    // let parse_game_events = decode_game_events_data(&mut archive, protocol);
    // let parsed_message_events = decode_message_events_data(&mut archive, protocol);
    let parsed_tracker_events = decode_tracker_events_data(&mut archive, protocol);

    // println!(
    //     "parsed game metadata: {:?}",
    //     decode_game_metadata_json(&mut archive)
    // );
    vec![]
}

fn load_mpq_archive(file_name: &str) -> Archive {
    Archive::open(file_name).expect("Failed to open MPQ archive")
}

fn decode_user_data(archive: &mut Archive, protocol: &Protocol) -> ParsedField {
    let user_data = archive
        .read_user_data()
        .expect("Failed to read User Data")
        .expect("Failed to unwrap User Data");
    let index: usize = protocol.replay_header_type_index.unwrap();
    let mut buffer = BitPackedBuff::new_big_endian(&user_data);

    versioned_decode("UserData", index, protocol, &mut buffer)
}

fn list_files_in_archive(archive: &mut Archive) {
    let listfile = archive.open_file("(listfile)").unwrap();
    let mut buf: Vec<u8> = vec![0; listfile.size() as usize];
    listfile.read(archive, &mut buf).unwrap();

    print!("{}", str::from_utf8(&buf).unwrap());
}

fn decode_details_data(archive: &mut Archive, protocol: &Protocol) -> ParsedField {
    let details_data_file = archive
        .open_file("replay.details")
        .expect("Failed to open replay.details file");
    let mut details_data: Vec<u8> = vec![0; details_data_file.size() as usize];
    details_data_file.read(archive, &mut details_data).unwrap();

    let index: usize = protocol.game_details_type_index.unwrap();
    let mut buffer = BitPackedBuff::new_big_endian(&details_data);

    versioned_decode("DetailsData", index, protocol, &mut buffer)
}

fn decode_init_data(archive: &mut Archive, protocol: &Protocol) -> ParsedField {
    let init_data_file = archive
        .open_file("replay.initdata")
        .expect("Failed to open replay.initdata file");
    let mut init_data: Vec<u8> = vec![0; init_data_file.size() as usize];
    init_data_file
        .read(archive, &mut init_data)
        .expect("Failed to read init data");

    let index: usize = protocol.replay_initdata_type_index.unwrap();
    let mut buffer = BitPackedBuff::new_big_endian(&init_data);

    raw_decode("InitData", index, protocol, &mut buffer)
}

fn decode_game_metadata_json(archive: &mut Archive) -> Value {
    let game_metadata_file = archive
        .open_file("replay.gamemetadata.json")
        .expect("Failed to open replay.gamemetadata.json file");
    let mut game_metadata: Vec<u8> = vec![0; game_metadata_file.size() as usize];
    game_metadata_file
        .read(archive, &mut game_metadata)
        .unwrap();

    println!("Game metadata: {:?}", game_metadata);
    serde_json::from_slice(&game_metadata).expect("Failed to parse JSON")
}

fn decode_game_events_data(archive: &mut Archive, protocol: &Protocol) {
    decode_events_data_for_variant(archive, protocol, EventTypeVariant::GameEvent);
}

fn decode_message_events_data(archive: &mut Archive, protocol: &Protocol) {
    decode_events_data_for_variant(archive, protocol, EventTypeVariant::MessageEvent);
}

fn decode_tracker_events_data(archive: &mut Archive, protocol: &Protocol) {
    decode_events_data_for_variant(archive, protocol, EventTypeVariant::TrackerEvent);
}

fn decode_events_data_for_variant(
    archive: &mut Archive,
    protocol: &Protocol,
    event_type_variant: EventTypeVariant,
) {
    let (events_data_file_name, event_id_type_index, event_types, user_id_present, decode): (
        &str,
        usize,
        &HashMap<u16, EventType>,
        bool,
        fn(
            name: &str,
            type_index: usize,
            protocol: &Protocol,
            buffer: &mut BitPackedBuff,
        ) -> ParsedField,
    ) = match event_type_variant {
        EventTypeVariant::GameEvent => (
            "replay.game.events",
            protocol.game_eventid_type_index.unwrap(),
            &protocol.game_event_types,
            true,
            raw_decode,
        ),
        EventTypeVariant::MessageEvent => (
            "replay.message.events",
            protocol.message_eventid_type_index.unwrap(),
            &protocol.message_event_types,
            true,
            raw_decode,
        ),
        EventTypeVariant::TrackerEvent => (
            "replay.tracker.events",
            protocol.tracker_eventid_type_index.unwrap(),
            &protocol.tracker_event_types,
            false,
            versioned_decode,
        ),
    };
    let events_data_file = archive
        .open_file(events_data_file_name)
        .expect("Failed to open events file");
    let mut events_data: Vec<u8> = vec![0; events_data_file.size() as usize];
    events_data_file.read(archive, &mut events_data).unwrap();

    let mut buffer = BitPackedBuff::new_big_endian(&events_data);
    let game_loop_type_index = protocol.game_loop_type_index.unwrap();
    let user_id_type_index = protocol.replay_userid_type_index.unwrap();
    let mut loop_id: usize = 0;
    while !buffer.done() {
        let loop_data = match decode("loopData", game_loop_type_index, protocol, &mut buffer) {
            ParsedField {
                name: _,
                value: Some(ParsedFieldType::Int(loop_data)),
            } => loop_data as usize,
            _ => panic!("Failed to parse game loop data in game events"),
        };
        println!("Loop Delta: {}", loop_data);
        loop_id += loop_data;

        let user_id;
        if user_id_present {
            let user_data_fields = match decode("userId", user_id_type_index, protocol, &mut buffer)
            {
                ParsedField {
                    name: _,
                    value: Some(ParsedFieldType::Struct(fields)),
                } => fields,
                _ => panic!("Failed to parse user ID in game events"),
            };
            user_id = match user_data_fields.iter().find(|f| f.name == "m_userId") {
                Some(ParsedField {
                    name: _,
                    value: Some(ParsedFieldType::Int(user_id)),
                }) => *user_id,
                _ => panic!("Failed to find user ID in user data"),
            };
            println!("User ID: {}", user_id);
        } else {
            // user_id = -1;
            println!("User ID is absent in Tracker Event");
        }

        let field = match decode("eventId", event_id_type_index, protocol, &mut buffer) {
            ParsedField {
                name: _,
                value: Some(field),
            } => field,
            _ => panic!("Failed to parse event ID in game events"),
        };
        println!("Event ID: {:?}", field);
        let event_id = match field {
            ParsedFieldType::Int(id) => id as u16,
            _ => panic!("Event ID is not an integer"),
        };
        let event_type = event_types
            .get(&event_id)
            .expect("Failed to get event type from protocol");
        let event = decode("eventData", event_type.type_index, protocol, &mut buffer);
        println!("Game Loop ID: {}", loop_id);
        println!("Event Data: {:?}", event);
        buffer.byte_align();
    }
}
