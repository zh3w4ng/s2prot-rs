pub mod buffer;
pub mod decoder;
pub mod types;

use crate::protocol::{self, types::Protocol};
use buffer::BitPackedBuff;
use decoder::Decoder;
use mpq::Archive;
use serde_json::Value;
use std::str;
use types::*;

pub fn build_replay(file_name: &str, protocol: &Protocol) -> Vec<ParsedField> {
    let mut archive = load_mpq_archive(file_name);
    let parsed_user_data = decode_user_data(&mut archive, protocol);
    list_files_in_archive(&mut archive);
    let parsed_details_data = decode_details_data(&mut archive, protocol);

    println!(
        "Parsed Game Metadata: {:?}",
        decode_game_metadata_json(&mut archive)
    );
    vec![parsed_details_data]
}

fn load_mpq_archive(file_name: &str) -> Archive {
    Archive::open(file_name).expect("Failed to open MPQ archive")
}

fn decode_user_data(archive: &mut Archive, protocol: &Protocol) -> ParsedField {
    let user_data = archive
        .read_user_data()
        // .unwrap()
        .expect("Failed to read User Data")
        .expect("Failed to unwrap User Data");
    let index: usize = protocol.replay_header_type_index.unwrap();
    let bit_packed_buffer = BitPackedBuff::new_big_endian(&user_data);

    match Decoder::new(bit_packed_buffer).decode("UserData", index, protocol) {
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

    match Decoder::new(bit_packed_buffer).decode("DetailsData", index, protocol) {
        Ok((_, parsed_field)) => parsed_field,
        _ => panic!("Failed to parse details data"),
    }
}

fn decode_game_metadata_json(archive: &mut Archive) -> Value {
    let game_metadata_file = archive
        .open_file("replay.gamemetadata.json")
        .expect("Failed to open replay.gamemetadata.jsonfile");
    let mut game_metadata: Vec<u8> = vec![0; game_metadata_file.size() as usize];
    game_metadata_file
        .read(archive, &mut game_metadata)
        .unwrap();

    println!("Game metadata: {:?}", game_metadata);
    serde_json::from_slice(&game_metadata).expect("Failed to parse JSON")
}
