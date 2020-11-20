use nom::{
    bytes::complete::is_not, character::complete::alpha1, character::complete::multispace0,
    character::complete::space1, error::make_error, error::ParseError, sequence::delimited,
    Err::Error as NomError, IResult,
};

pub struct Dockerfile<'a> {
    pub instructions: Vec<Instruction<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Instruction<'a> {
    From(From<'a>),
    Run(Run<'a>),
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

/// Parse a single instruction.
///
/// Given an input string, parses the first instruction encountered.
/// ```rust
/// # use dockerfile_parser::{parse_instruction, Instruction, From, Run};
/// # use nom::{
/// #     bytes::complete::is_not, bytes::complete::tag, character::complete::multispace0,
/// #     character::complete::space1, error::ParseError, sequence::delimited, IResult,
/// # };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
/// FROM ubuntu:test
/// RUN /bin/bash -c echo "test"
/// "#;
///
/// let (rem, from_instruction) = parse_instruction(input)?;
/// let (_, run_instruction) = parse_instruction(rem)?;
///
/// match (from_instruction, run_instruction) {
///     (Instruction::From(from), Instruction::Run(run)) => {
///         assert_eq!(from.image, "ubuntu:test");
///         assert_eq!(run.command, r#"/bin/bash -c echo "test""#);
///     }
///     _ => panic!("Didn't parse instructions correctly!"),
/// }
/// # Ok(())
/// # }
/// ```
pub fn parse_instruction(input: &str) -> IResult<&str, Instruction<'_>> {
    let (rem, instruction): (&str, &str) = delimited(multispace0, alpha1, space1)(input)?;
    // instruction names are all ASCII (are they???), this is much faster than `to_lowercase()`.
    match instruction.to_ascii_lowercase().as_str() {
        <From>::NAME => Ok(<From>::parse(rem)?),
        <Run>::NAME => Ok(<Run>::parse(rem)?),
        _ => Err(NomError(make_error(rem, nom::error::ErrorKind::Tag))),
    }
}

trait InstructionParser {
    const NAME: &'static str;

    fn parse(input: &str) -> IResult<&str, Instruction<'_>>;
}

impl InstructionParser for From<'_> {
    const NAME: &'static str = "from";

    fn parse(input: &str) -> IResult<&str, Instruction<'_>> {
        let (rem, image) = ws(is_not_newline())(input)?;
        Ok((rem, Instruction::From(From::new(image))))
    }
}

impl InstructionParser for Run<'_> {
    const NAME: &'static str = "run";

    fn parse(input: &str) -> IResult<&str, Instruction<'_>> {
        let (rem, image) = ws(is_not_newline())(input)?;
        Ok((rem, Instruction::Run(Run::new(image))))
    }
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

    //    proptest! {
    //         #[test]
    //         fn from_instruction_parses_correctly((from_instruction, expected_image) in arbitrary_from()) {
    //            let result = from(&from_instruction).unwrap();
    //            assert_eq!(
    //                result.1,
    //                From{
    //                    image: &expected_image
    //                }
    //            );
    //            assert_eq!(result.0, "");
    //        }
    //    }
}
