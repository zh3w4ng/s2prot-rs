pub mod protocol;
pub mod replay;
use std::path::Path;

pub mod bit_packed_buff {
    pub fn read_bits() {
        println!("hello");
    }
}

use protocol::types::Protocol;
use std::fs;

pub fn load_protocol_version(version: &str) -> Protocol {
    let folder = Path::new("assets/protocols");
    let file_path = format!("{}/protocol{}.py", folder.display(), version);
    let content = fs::read_to_string(file_path).expect("Failed to read file {file_path}");
    let (_, protocol) =
        protocol::build_protocol(content.as_str()).expect("Failed to build protocol");

    protocol
}

pub fn load_replay_file(file_name: &str, protocol: &Protocol) {
    let file_path = format!("assets\\replays\\{}", file_name);
    let replay = replay::build_replay(&file_path, protocol);
    println!("{:?}", replay);
}
