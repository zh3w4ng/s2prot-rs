pub mod buffer;
pub mod decoders;
pub mod types;

use crate::protocol::{self, types::Protocol};
use buffer::BitPackedBuff;
use decoders::{BitPackedDecoder, VersionedDecoder};
use mpq::Archive;
use serde_json::Value;
use std::str;
use types::*;

pub fn build_replay(file_name: &str, protocol: &Protocol) -> Vec<ParsedField> {
    let mut archive = load_mpq_archive(file_name);
    // let parsed_user_data = decode_user_data(&mut archive, protocol);
    list_files_in_archive(&mut archive);
    // let parsed_details_data = decode_details_data(&mut archive, protocol);
    // let parsed_init_data = decode_init_data(&mut archive, protocol);
    let parsed_data = decode_game_events_data(&mut archive, protocol);

    // println!(
    //     "Parsed Game Metadata: {:?}",
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
    let bit_packed_buffer = BitPackedBuff::new_big_endian(&user_data);

    match VersionedDecoder::new(bit_packed_buffer).decode("UserData", index, protocol) {
        Ok((_, parsed_field)) => parsed_field,
        _ => panic!("Failed to parse user data"),
    }
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
    let bit_packed_buffer = BitPackedBuff::new_big_endian(&details_data);

    match VersionedDecoder::new(bit_packed_buffer).decode("DetailsData", index, protocol) {
        Ok((_, parsed_field)) => parsed_field,
        _ => panic!("Failed to parse details data"),
    }
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
    let bit_packed_buffer = BitPackedBuff::new_big_endian(&init_data);

    match BitPackedDecoder::new(bit_packed_buffer).decode("InitData", index, &protocol.type_infos) {
        Ok((_, parsed_field)) => parsed_field,
        _ => panic!("Failed to parse init data"),
    }
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
    let game_events_data_file = archive
        .open_file("replay.game.events")
        .expect("Failed to open replay.game.events file");
    let mut game_events_data: Vec<u8> = vec![0; game_events_data_file.size() as usize];
    game_events_data_file
        .read(archive, &mut game_events_data)
        .unwrap();

    let bit_packed_buffer = BitPackedBuff::new_big_endian(&game_events_data);
    let mut decoder = BitPackedDecoder::new(bit_packed_buffer);
    let game_loop_type_index = protocol.game_loop_type_index.unwrap();
    let user_id_type_index = protocol.replay_userid_type_index.unwrap();
    let event_id_type_index = protocol.game_eventid_type_index.unwrap();
    let mut loop_id: usize = 0;
    while !decoder.completed() {
        let loop_data = match decoder.decode("loopData", game_loop_type_index, &protocol.type_infos)
        {
            Ok((
                _,
                ParsedField {
                    name: _,
                    value: Some(ParsedFieldType::Int(loop_data)),
                },
            )) => loop_data as usize,
            _ => panic!("Failed to parse game loop data in game events"),
        };
        println!("Loop Delta: {}", loop_data);
        loop_id += loop_data;

        let user_data_fields =
            match decoder.decode("userId", user_id_type_index, &protocol.type_infos) {
                Ok((
                    _,
                    ParsedField {
                        name: _,
                        value: Some(ParsedFieldType::Struct(fields)),
                    },
                )) => fields,
                _ => panic!("Failed to parse user ID in game events"),
            };
        let user_id = match user_data_fields.iter().find(|f| f.name == "m_userId") {
            Some(ParsedField {
                name: _,
                value: Some(ParsedFieldType::Int(user_id)),
            }) => user_id,
            _ => panic!("Failed to find user ID in user data"),
        };
        println!("User ID: {}", user_id);

        let field = match decoder.decode("eventId", event_id_type_index, &protocol.type_infos) {
            Ok((
                _,
                ParsedField {
                    name: _,
                    value: Some(field),
                },
            )) => field,
            _ => panic!("Failed to parse event ID in game events"),
        };
        println!("Event ID: {:?}", field);
        let event_id = match field {
            ParsedFieldType::Int(id) => id as u16,
            _ => panic!("Event ID is not an integer"),
        };
        let event_type = protocol
            .game_event_types
            .get(&event_id)
            .expect("Failed to get event type from protocol");
        let event = match decoder.decode("eventData", event_type.type_index, &protocol.type_infos) {
            Ok((_, parsed_field)) => parsed_field,
            _ => panic!("Failed to parse event data in game events"),
        };
        println!("Game Loop ID: {}", loop_id);
        println!("Event Data: {:?}", event);
        decoder.align();
    }
}
