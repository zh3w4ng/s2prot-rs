extern crate nom;

use nom::{
    bytes::complete::{tag, take_till, take_until},
    character::complete::{line_ending, newline},
    multi::many0,
    sequence::preceded,
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
    // let (input, _) = tag(" import ")(input)?;
    let (input, imported) = preceded(tag(" import "), take_until("\n"))(input)?;

    Ok((input, (import_from, imported.trim_end())))
}

pub fn parse_blank_lines(input: &str) -> IResult<&str, &str> {
    let (input, _) = many0(line_ending)(input)?;

    Ok((input, ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_comments_with_no_error() {
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
}
