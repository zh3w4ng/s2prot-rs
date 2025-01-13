pub mod parsers;
pub mod types;

use nom::IResult;
use parsers::{parse_offset_and_length, parse_type_name};
use types::TypeInfo;

pub fn build_type_info(input: &str) -> IResult<&str, TypeInfo> {
    let (input, _) = parse_type_name(input)?;
    let (input, (offset, length)) = parse_offset_and_length(input)?;

    Ok((input, TypeInfo::Int { offset, length }))
}

#[test]
fn it_create_an_int_type_info() {
    let input = "('_int',[(0,7)]),  #0";
    let Ok((_, type_info)) = build_type_info(input) else {
        panic!("Failed to build type info.");
    };
    assert_eq!(
        type_info,
        TypeInfo::Int {
            offset: 0,
            length: 7
        }
    );
}
