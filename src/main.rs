use s2prot_rs::load_protocol_version;
use s2prot_rs::load_replay_file;

fn main() {
    let protocol = load_protocol_version("93272");
    load_replay_file("test.SC2Replay", &protocol);
}
