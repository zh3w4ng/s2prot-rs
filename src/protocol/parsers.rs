mod line_parsers;

use std::collections::HashMap;

use super::types::{EventType, Field, TypeInfo};
use line_parsers::{
    parse_choice_fields, parse_constant, parse_event_type, parse_offset_and_length,
    parse_struct_fields, parse_type_index, parse_type_name, skip_remaining_of_line,
};
use nom::IResult;

pub fn skip_current_line(input: &str) -> IResult<&str, &str> {
    let (input, _) = skip_remaining_of_line(input)?;
    Ok((input, ""))
}

pub fn build_constant(input: &str) -> IResult<&str, Option<usize>> {
    let (input, constant) = parse_constant(input)?;
    Ok((input, Some(constant)))
}

pub fn build_type_infos(mut input: &str) -> IResult<&str, Vec<TypeInfo>> {
    let mut vec = Vec::new();
    (input, _) = skip_remaining_of_line(input)?;
    while !input.starts_with("]") {
        match build_type_info(input) {
            Ok((rest, type_info)) => {
                vec.push(type_info);
                input = rest;
            }
            _ => panic!("Failed to build type info"),
        }
    }
    (input, _) = skip_remaining_of_line(input)?;
    Ok((input, vec))
}

fn build_type_info(input: &str) -> IResult<&str, TypeInfo> {
    let (input, type_name) = parse_type_name(input)?;
    let (input, res) = match type_name {
        "_bool" => {
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::Bool)
        }
        "_optional" => {
            let (input, type_index) = parse_type_index(input)?;
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::Optional { type_index })
        }
        "_int" => {
            let (input, (offset, length)) = parse_offset_and_length(input)?;
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::Int { offset, length })
        }
        "_blob" => {
            let (input, (offset, length)) = parse_offset_and_length(input)?;
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::Blob { offset, length })
        }
        "_array" => {
            let (input, (offset, length)) = parse_offset_and_length(input)?;
            let (input, type_index) = parse_type_index(input)?;
            let (input, _) = skip_remaining_of_line(input)?;
            (
                input,
                TypeInfo::Array {
                    offset,
                    length,
                    type_index,
                },
            )
        }
        "_bitarray" => {
            let (input, (offset, length)) = parse_offset_and_length(input)?;
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::BitArray { offset, length })
        }
        "_choice" => {
            let (input, (offset, length)) = parse_offset_and_length(input)?;
            let (input, fields) = parse_choice_fields(input)?;
            let fields = fields
                .iter()
                .map(|(name, index, tag)| Field {
                    name: name.to_string(),
                    type_index: *index,
                    tag: *tag,
                })
                .collect();
            let (input, _) = skip_remaining_of_line(input)?;
            (
                input,
                TypeInfo::Choice {
                    offset,
                    length,
                    fields,
                },
            )
        }
        "_struct" => {
            let (input, fields) = parse_struct_fields(input)?;
            let mut fields: Vec<Field> = fields
                .iter()
                .map(|(name, index, tag)| Field {
                    name: name.to_string(),
                    type_index: *index,
                    tag: *tag,
                })
                .collect();
            let (input, _) = skip_remaining_of_line(input)?;
            fields.sort_by(|f1, f2| f1.tag.cmp(&f2.tag));
            (input, TypeInfo::Struct { fields })
        }
        "_fourcc" => {
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::FourCC)
        }
        "_null" => {
            let (input, _) = skip_remaining_of_line(input)?;
            (input, TypeInfo::Null)
        }
        _ => unimplemented!(),
    };

    Ok((input, res))
}

pub fn build_event_types(input: &str) -> IResult<&str, HashMap<u16, EventType>> {
    let mut event_types = HashMap::new();
    let mut input = input;
    (input, _) = skip_remaining_of_line(input)?;
    while !input.starts_with("}") {
        match build_event_type(input) {
            Ok((rest, event_type)) => {
                event_types.insert(event_type.event_id, event_type);
                input = rest;
            }
            _ => panic!("Failed to build event_type info: {input}"),
        }
    }
    (input, _) = skip_remaining_of_line(input)?;
    Ok((input, event_types))
}

fn build_event_type(input: &str) -> IResult<&str, EventType> {
    let (input, (event_id, event_name, type_index)) = parse_event_type(input)?;
    let (input, _) = skip_remaining_of_line(input)?;
    Ok((
        input,
        EventType {
            event_id,
            type_index,
            event_name: event_name.to_string(),
        },
    ))
}

#[test]
fn it_build_type_infos_with_no_error() {
    let input = r#"typeinfos = [
                        ('_int',[(0,7)]),  #0
                        ('_int',[(0,4)]),  #1
                        ('_bool',[]),  #13
                        ('_blob',[(0,8)]),  #9
                        ('_bitarray',[(0,9)]),  #102
                        ('_array',[(16,0),10]),  #14
                        ('_optional',[84]),  #146
                        ('_choice',[(0,2),{0:('m_uint6',3),1:('m_uint14',4),2:('m_uint22',5),3:('m_uint32',6)}]),  #7
                        ('_struct',[[('m_dataDeprecated',15,0),('m_data',16,1)]]),  #17
                        ('_fourcc',[]),  #19
                        ('_null',[]),  #94
]
"#;
    let Ok((input, vec)) = build_type_infos(input) else {
        panic!("Failed to build type infos: {input}")
    };
    assert_eq!(
        vec,
        [
            TypeInfo::Int {
                offset: 0,
                length: 7
            },
            TypeInfo::Int {
                offset: 0,
                length: 4
            },
            TypeInfo::Bool,
            TypeInfo::Blob {
                offset: 0,
                length: 8
            },
            TypeInfo::BitArray {
                offset: 0,
                length: 9
            },
            TypeInfo::Array {
                offset: 16,
                length: 0,
                type_index: 10
            },
            TypeInfo::Optional { type_index: 84 },
            TypeInfo::Choice {
                offset: 0,
                length: 2,
                fields: vec![
                    Field {
                        name: "m_uint6".to_string(),
                        type_index: 3,
                        tag: 0
                    },
                    Field {
                        name: "m_uint14".to_string(),
                        type_index: 4,
                        tag: 1
                    },
                    Field {
                        name: "m_uint22".to_string(),
                        type_index: 5,
                        tag: 2
                    },
                    Field {
                        name: "m_uint32".to_string(),
                        type_index: 6,
                        tag: 3
                    }
                ]
            },
            TypeInfo::Struct {
                fields: vec![
                    Field {
                        name: "m_dataDeprecated".to_string(),
                        type_index: 15,
                        tag: 0
                    },
                    Field {
                        name: "m_data".to_string(),
                        type_index: 16,
                        tag: 1
                    }
                ]
            },
            TypeInfo::FourCC,
            TypeInfo::Null
        ]
    );
    assert_eq!(input, "");
}

#[test]
fn it_build_event_types_with_no_error() {
    let input = r#"message_event_types = {
                    0: (192, 'NNet.Game.SChatMessage'),
                    1: (193, 'NNet.Game.SPingMessage'),
}
"#;
    let Ok((input, event_types)) = build_event_types(input) else {
        panic!("Failed to build event types: {input}")
    };
    assert_eq!(
        event_types,
        vec![
            (
                0,
                EventType {
                    event_id: 0,
                    type_index: 192,
                    event_name: "NNet.Game.SChatMessage".to_string(),
                }
            ),
            (
                1,
                EventType {
                    event_id: 1,
                    type_index: 193,
                    event_name: "NNet.Game.SPingMessage".to_string(),
                }
            )
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(input, "");
}
