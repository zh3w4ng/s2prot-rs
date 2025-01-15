extern crate nom;

use nom::{
    bytes::complete::{is_a, tag, take, take_till, take_until},
    character::complete::{line_ending, newline, space0},
    multi::many0,
    sequence::{delimited, preceded, terminated},
    IResult,
};

pub fn parse_comments(input: &str) -> IResult<&str, &str> {
    let (input, _) = many0(line_ending)(input)?;
    let (input, _) = tag("#")(input)?;
    let (input, comment) = take_till(|c| c == '\n')(input)?;
    let (input, _) = newline(input)?;

    Ok((input, comment.trim_end()))
}

pub fn parse_imports(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, import_from) = preceded(tag("from "), take_until(" import "))(input)?;
    let (input, imported) = preceded(tag(" import "), take_until("\n"))(input)?;

    Ok((input, (import_from, imported.trim_end())))
}

pub fn parse_blank_lines(input: &str) -> IResult<&str, &str> {
    let (input, _) = many0(line_ending)(input)?;

    Ok((input, ""))
}

pub fn parse_type_name(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, type_name) = delimited(tag("('"), take_until("'"), tag("',"))(input)?;

    Ok((input, type_name))
}

pub fn parse_offset_and_length(input: &str) -> IResult<&str, (usize, usize)> {
    let (input, offset) = delimited(tag("[("), take_until(","), tag(","))(input)?;
    let (input, length) = take_until(")")(input)?;
    let offset: usize = offset.parse().expect("Not a valid number");
    let length: usize = length.parse().expect("Not a valid number");

    Ok((input, (offset, length)))
}

pub fn parse_type_index(input: &str) -> IResult<&str, u16> {
    let (input, type_index) = delimited(is_a("),["), take_until("]"), tag("]"))(input)?;
    let type_index: u16 = type_index.parse().expect("Not a valid number");

    Ok((input, type_index))
}

pub fn parse_choice_fields(input: &str) -> IResult<&str, Vec<(&str, u16)>> {
    let mut fields: Vec<(&str, u16)> = Vec::new();
    let mut field_name: &str;
    let mut field_type_index: &str;
    let (mut input, _) = tag("),")(input)?;
    // {0:('m_uint6',3),1:('m_uint14',4),2:('m_uint22',5),3:('m_uint32',6)}]),  #7
    while !input.is_empty() && !input.starts_with("}") {
        (input, _) = delimited(is_a(",{"), take_until("'"), tag("'"))(input)?;
        (input, field_name) = terminated(take_until("',"), tag("',"))(input)?;
        (input, field_type_index) = terminated(take_until(")"), tag(")"))(input)?;
        let field_type_index = field_type_index.parse().expect("Not a valid number");

        fields.push((field_name, field_type_index));
    }

    Ok((input, fields))
}

pub fn parse_struct_fields(input: &str) -> IResult<&str, Vec<(&str, u16)>> {
    // [[('m_dataDeprecated',15,0),('m_data',16,1)]]),  #17
    let mut fields: Vec<(&str, u16)> = Vec::new();
    let mut field_name: &str;
    let mut field_type_index: &str;
    let mut input = input;
    while !input.is_empty() && !input.starts_with("]") {
        (input, _) = delimited(is_a(",["), take_until("'"), tag("'"))(input)?;
        (input, field_name) = terminated(take_until("',"), tag("',"))(input)?;
        (input, field_type_index) = terminated(take_until(","), tag(","))(input)?;
        (input, _) = terminated(take_until(")"), tag(")"))(input)?;
        let field_type_index = field_type_index.parse().expect("Not a valid number");

        fields.push((field_name, field_type_index));
    }

    Ok((input, fields))
}

pub fn skip_remaining_of_line(input: &str) -> IResult<&str, &str> {
    let (input, _) = preceded(take_until("\n"), newline)(input)?;

    Ok((input, ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parse_comments_with_no_error() {
        let input = r#"
# Copyright (c) 2015-2017 Blizzard Entertainment



# Decoding instructions for each protocol type.
typeinfos = [
    ('_int',[(0,7)]),  #0
    ('_int',[(0,4)]),  #1
    ('_int',[(0,5)]),  #2
    ('_int',[(0,6)]),  #3
]
        "#;
        let Ok((_, comment)) = parse_comments(input) else {
            panic!("parse_comments failed.")
        };
        assert_eq!(comment, " Copyright (c) 2015-2017 Blizzard Entertainment");
    }

    #[test]
    fn it_parse_imports_with_no_error() {
        let input = r#"from s2protocol.decoders import *
        "#;
        let Ok((_, (import_from, imported))) = parse_imports(input) else {
            panic!("parse_imports failed.")
        };
        assert_eq!(import_from, "s2protocol.decoders");
        assert_eq!(imported, "*");
    }

    #[test]
    fn it_parse_blank_lines_with_no_error() {
        let input = r#"

abc"#;
        let Ok((input, res)) = parse_blank_lines(input) else {
            panic!("parse_blank_lines failed.")
        };
        assert_eq!(input, "abc");
        assert_eq!(res, "");
    }

    #[test]
    fn it_parse_type_name_with_no_error() {
        let input = "    ('_int',[(0,7)]),  #0";
        let Ok((input, type_name)) = parse_type_name(input) else {
            panic!("parse_type_name failed.")
        };
        assert_eq!(type_name, "_int");
        assert_eq!(input, "[(0,7)]),  #0");
    }

    #[test]
    fn it_parse_offset_and_length_with_no_error() {
        let input = "[(0,7)]),  #0";
        let Ok((input, (offset, length))) = parse_offset_and_length(input) else {
            panic!("parse_offset_and_length failed.")
        };
        assert_eq!(offset, 0);
        assert_eq!(length, 7);
        assert_eq!(input, ")]),  #0");
    }

    #[test]
    fn it_parse_optional_type_index_with_no_error() {
        let input = "[10])";
        let Ok((input, type_index)) = parse_type_index(input) else {
            panic!("parse_optional_type_index failed.")
        };
        assert_eq!(type_index, 10);
        assert_eq!(input, ")");
    }

    #[test]
    fn it_parse_array_type_index_with_no_error() {
        let input = "),10])";
        let Ok((input, type_index)) = parse_type_index(input) else {
            panic!("parse_array_type_index failed.")
        };
        assert_eq!(type_index, 10);
        assert_eq!(input, ")");
    }

    #[test]
    fn it_parse_choice_fields_with_no_error() {
        let input = "),{0:('m_uint6',3),1:('m_uint14',4),2:('m_uint22',5),3:('m_uint32',6)}]),  #7";
        let Ok((input, fields)) = parse_choice_fields(input) else {
            panic!("it_parse_choice_fields failed.")
        };
        assert_eq!(
            fields,
            vec![
                ("m_uint6", 3),
                ("m_uint14", 4),
                ("m_uint22", 5),
                ("m_uint32", 6)
            ]
        );
        assert_eq!(input, "}]),  #7")
    }

    #[test]
    fn it_parse_struct_fields_with_no_error() {
        let input = "[[('m_dataDeprecated',15,0),('m_data',16,1)]]),  #17";
        let Ok((input, vec)) = parse_struct_fields(input) else {
            panic!("parse_struct_fields failed.")
        };
        assert_eq!(vec, vec![("m_dataDeprecated", 15), ("m_data", 16)]);
        assert_eq!(input, "]]),  #17");
    }

    #[test]
    fn it_skip_remaining_of_line_with_no_error() {
        let input = r#"),  #14
"#;
        let Ok((input, _)) = skip_remaining_of_line(input) else {
            panic!("skip_remaining_of_line failed.")
        };
        assert_eq!(input, "");
    }
}
