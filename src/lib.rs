use nom::{
    bytes::complete::is_not, bytes::complete::tag, character::complete::multispace0,
    character::complete::space1, error::ParseError, sequence::delimited, IResult,
};

pub struct Dockerfile<'a> {
    pub instructions: Vec<Instruction<'a>>,
}

pub enum Instruction<'a> {
    From(From<'a>),
}

#[derive(Debug, PartialEq)]
pub struct From<'a> {
    pub image: &'a str,
}

impl<'a> From<'a> {
    pub fn new(image: &'a str) -> Self {
        Self { image }
    }
}

#[derive(Debug, PartialEq)]
pub struct Run<'a> {
    pub command: &'a str,
}

impl<'a> Run<'a> {
    pub fn new(command: &'a str) -> Self {
        Self { command }
    }
}

const FROM_INSTRUCTION: &str = "FROM";
const RUN_INSTRUCTION: &str = "RUN";

pub fn from(s: &str) -> IResult<&str, From<'_>> {
    let (rem, image) = space_wrapped_instruction(tag(FROM_INSTRUCTION))(s)
        .and_then(|(rem, _)| ws(is_not_space())(rem))?;
    Ok((rem, From::new(image)))
}

pub fn run(s: &str) -> IResult<&str, Run<'_>> {
    let (rem, command) = space_wrapped_instruction(tag(RUN_INSTRUCTION))(s)
        .and_then(|(rem, _)| ws(is_not_newline())(rem))?;
    Ok((rem, Run::new(command)))
}

fn space_wrapped_instruction<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, space1)
}

fn is_not_space<'a, E: ParseError<&'a str>>() -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
{
    is_not(" \n\r\t")
}

fn is_not_newline<'a, E: ParseError<&'a str>>(
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E> {
    is_not("\r\n")
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use crate::{from, From};
    use const_format::formatcp;
    use proptest::prelude::*;

    const DOMAIN_AND_PORT_REGEX: &str = r#"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]:[0-9]{0,5}/)"#;
    const IMAGE_NAME_REGEX: &str = formatcp!(
        "({}?([a-z0-9]+[._-]?)+[a-z0-9]+:([a-z0-9]+[._-]?)+[a-z0-9]+",
        DOMAIN_AND_PORT_REGEX
    );

    /// Generates a FROM instruction.
    fn arbitrary_from() -> impl Strategy<Value = (String, String)> {
        proptest::string::string_regex(IMAGE_NAME_REGEX)
            .expect("failed to generate strategy")
            .prop_map(|s| (format!("FROM {}", s), s))
            .boxed()
    }

    proptest! {
         #[test]
         fn from_instruction_parses_correctly((from_instruction, expected_image) in arbitrary_from()) {
            let result = from(&from_instruction).unwrap();
            assert_eq!(
                result.1,
                From{
                    image: &expected_image
                }
            );
            assert_eq!(result.0, "");
        }
    }
}
