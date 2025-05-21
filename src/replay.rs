pub mod buffer;
pub mod decoder;
pub mod types;

use crate::protocol::types::Protocol;
use buffer::BitPackedBuff;
use decoder::Decoder;
use mpq::Archive;
use types::*;

pub fn build_replay(file_name: &str, protocol: &Protocol) -> ParsedField {
    let mut archive = load_mpq_archive(file_name);
    let user_data = archive
        .read_user_data()
        .unwrap()
        .expect("Failed to retrieve User Data");
    let index: usize = protocol.replay_header_type_index.unwrap();
    let bit_packed_buffer = BitPackedBuff::new_big_endian(&user_data);
    match Decoder::new(bit_packed_buffer).decode("UserData", index, protocol) {
        Ok((_, parsed_field)) => parsed_field,
        _ => panic!("Failed to parse user data"),
    }
}

pub fn load_mpq_archive(file_name: &str) -> Archive {
    Archive::open(file_name).expect("Failed to open MPQ archive")
}
