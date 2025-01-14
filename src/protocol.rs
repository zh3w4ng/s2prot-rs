pub mod parsers;
pub mod types;

use nom::IResult;
use parsers::{
    parse_offset_and_length, parse_offset_and_length_and_fields,
    parse_offset_and_length_and_type_index, parse_type_index, parse_type_name,
    skip_remaining_of_line,
};
use types::{Field, TypeInfo};

pub fn build_type_info(input: &str) -> IResult<&str, TypeInfo> {
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
            let (input, (offset, length, type_index)) =
                parse_offset_and_length_and_type_index(input)?;
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
        "_choice" => {
            let (input, (offset, length, fields)) = parse_offset_and_length_and_fields(input)?;
            let fields = fields
                .iter()
                .map(|(name, index)| Field {
                    name: name.to_string(),
                    type_index: *index,
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
        _ => unimplemented!(),
    };

    Ok((input, res))
}

#[test]
fn it_build_type_infos_with_no_error() {
    let mut input = r#"     ('_int',[(0,7)]),  #0
                        ('_int',[(0,4)]),  #1
                        ('_bool',[]),  #13
                        ('_blob',[(0,8)]),  #9
                        ('_array',[(16,0),10]),  #14
                        ('_optional',[84]),  #146
                        ('_choice',[(0,2),{0:('m_uint6',3),1:('m_uint14',4),2:('m_uint22',5),3:('m_uint32',6)}]),  #7
"#;
    let mut vec: Vec<TypeInfo> = Vec::with_capacity(6);
    while !input.is_empty() {
        match build_type_info(input) {
            Ok((rest, type_info)) => {
                vec.push(type_info);
                input = rest;
            }
            _ => panic!("Failed to build type info: {input}"),
        }
    }
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
                        type_index: 3
                    },
                    Field {
                        name: "m_uint14".to_string(),
                        type_index: 4
                    },
                    Field {
                        name: "m_uint22".to_string(),
                        type_index: 5
                    },
                    Field {
                        name: "m_uint32".to_string(),
                        type_index: 6
                    }
                ]
            }
        ]
    );
}
