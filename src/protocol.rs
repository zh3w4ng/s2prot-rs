mod parsers;
mod types;

use nom::IResult;
use std::collections::HashMap;

pub fn build_protocol(mut input: &str) -> IResult<&str, types::Protocol> {
    // todo!()

    let mut type_infos = Vec::new();
    let mut game_event_types = HashMap::new();
    let mut message_event_types = HashMap::new();
    let mut tracker_event_types = HashMap::new();
    let mut game_loop_type_index = None;
    let mut game_eventid_type_index = None;
    let mut message_eventid_type_index = None;
    let mut replay_userid_type_index = None;
    let mut replay_header_type_index = None;
    let mut game_details_type_index = None;
    let mut replay_initdata_type_index = None;
    let mut tracker_eventid_type_index = None;
    let build_version = 93272;
    let has_tracker_events = false;

    while !input.starts_with("def") {
        if input.starts_with("typeinfos") {
            (input, type_infos) = parsers::build_type_infos(input)?;
        } else if input.starts_with("game_event_types") {
            (input, game_event_types) = parsers::build_event_types(input)?;
        } else if input.starts_with("message_event_types") {
            (input, message_event_types) = parsers::build_event_types(input)?;
        } else if input.starts_with("tracker_event_types") {
            (input, tracker_event_types) = parsers::build_event_types(input)?;
        } else if input.starts_with("#")
            || input.starts_with("from")
            || input.starts_with("\r")
            || input.starts_with("\n")
        {
            (input, _) = parsers::skip_current_line(input)?;
        } else if input.starts_with("tracker_eventid") {
            (input, tracker_eventid_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("svaruint32") {
            (input, game_loop_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("game_eventid") {
            (input, game_eventid_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("message_eventid") {
            (input, message_eventid_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("replay_userid") {
            (input, replay_userid_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("game_details") {
            (input, game_details_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("replay_header") {
            (input, replay_header_type_index) = parsers::build_constant(input)?;
        } else if input.starts_with("replay_initdata") {
            (input, replay_initdata_type_index) = parsers::build_constant(input)?;
        } else {
            panic!("Unknown line: {input}");
        }
    }

    Ok((
        "",
        types::Protocol {
            build_version,
            has_tracker_events,
            game_eventid_type_index,
            message_eventid_type_index,
            tracker_eventid_type_index,
            game_loop_type_index,
            replay_userid_type_index,
            replay_header_type_index,
            game_details_type_index,
            replay_initdata_type_index,
            tracker_event_types,
            type_infos,
            game_event_types,
            message_event_types,
        },
    ))
}
