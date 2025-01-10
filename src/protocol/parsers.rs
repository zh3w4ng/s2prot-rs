extern crate nom;

use nom::{
    IResult,
    bytes::complete::{
        tag,
        take_till
    },
    character::{
        is_newline,
        complete::line_ending,
    },
    multi::many0
};

pub fn parse_comments(input: &str) -> IResult<&str, &str> {
    let (input, _) = many0(line_ending)(input)?;
    let (input, _) = tag("#")(input)?;
    let (input, comment) = take_till(|c| c == '\n')(input)?;
    //let (input, _) = newline(input)?;

    Ok((input, comment))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_comments_with_no_error() {
        let input = r#"
# Copyright (c) 2015-2017 Blizzard Entertainment
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.
        "#;
        let Ok((_, comment)) = parse_comments(input) else { panic!("parse_comments failed.")};
        assert_eq!(comment.trim_end(), " Copyright (c) 2015-2017 Blizzard Entertainment");
        //assert_eq!(rest.len(), 100);
    }
}
