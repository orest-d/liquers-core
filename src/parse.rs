#[macro_use]
use nom;

extern crate nom_locate;
use nom_locate::LocatedSpan;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1, take_while_m_n};
use nom::character::complete::digit1;
use nom::character::{is_alphabetic, is_alphanumeric, is_hex_digit};
use nom::combinator::{cut, opt};
use nom::multi::{many0, many1_count, many1, separated_list, separated_nonempty_list};
use nom::sequence::pair;
use nom::*;

use percent_encoding::{percent_decode_str, PercentDecode};

use crate::error::Error;
use crate::query::{ActionParameter, ActionRequest, Position, Query, QuerySegment, SegmentHeader};

type Span<'a> = LocatedSpan<&'a str>;

impl<'a> From<Span<'a>> for Position {
    fn from(span: Span<'a>) -> Position {
        Position {
            offset: span.location_offset(),
            line: span.location_line(),
            column: span.get_utf8_column(),
        }
    }
}

fn identifier(text: Span) -> IResult<Span, String> {
    let (text, a) = take_while1(|c: char| c.is_alphabetic() || c == '_')(text)?;
    let (text, b) = take_while(|c: char| c.is_alphanumeric() || c == '_')(text)?;

    Ok((text, format!("{}{}", a, b)))
}

fn parameter_text(text: Span) -> IResult<Span, String> {
    let (text, par) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(text)?;
    Ok((text, format!("{}", par)))
}

fn percent_encoding(text: Span) -> IResult<Span, String> {
    let (text, _percent) = tag("%")(text)?;
    let (text, hex) = cut(take_while_m_n(2, 2, |c: char| c.is_digit(16)))(text)?;
    Ok((text, format!("%{}", hex)))
}

fn tilde_entity(text: Span) -> IResult<Span, String> {
    let (text, _tilde) = tag("~")(text)?;
    Ok((text, "~".to_owned()))
}

fn minus_entity(text: Span) -> IResult<Span, String> {
    let (text, _tilde) = tag("_")(text)?;
    Ok((text, "-".to_owned()))
}

fn negative_number_entity(text: Span) -> IResult<Span, String> {
    let (text, number) = digit1(text)?;
    Ok((text, format!("-{}", number)))
}

fn parameter_entity(text: Span) -> IResult<Span, String> {
    let (text, _start) = tag("~")(text)?;
    let position: Position = text.into();
    let (text, entity) = cut(alt((tilde_entity, minus_entity, negative_number_entity)))(text)?;
    Ok((text, format!("{}", entity)))
}

fn parameter(text: Span) -> IResult<Span, ActionParameter> {
    let position: Position = text.into();
    let (text, par) = many0(alt((parameter_text, parameter_entity, percent_encoding)))(text)?;
    //    let err: nom::Err<(Span, nom::error::ErrorKind)> = nom::error::make_error(text, nom::error::ErrorKind::Escaped);
    let par = par.join("");
    let par = percent_decode_str(&par).decode_utf8().map_err(|e| {
        nom::Err::Failure(nom::error::ParseError::from_error_kind(
            text,
            nom::error::ErrorKind::Escaped,
        ))
    })?;

    Ok((text, ActionParameter::new_parsed(par.to_string(), position)))
}

fn parse_action(text: Span) -> IResult<Span, ActionRequest> {
    let position: Position = text.into();
    let (text, name) = identifier(text)?;
    let (text, p) = many0(pair(tag("-"), parameter))(text)?;

    Ok((
        text,
        ActionRequest {
            name: name,
            position,
            parameters: p.iter().map(|x| x.1.clone()).collect(),
        },
    ))
}

fn parse_action_path(text: Span) -> IResult<Span, Vec<ActionRequest>> {
    separated_list(tag("/"), parse_action)(text)
}

fn parse_action_path_nonempty(text: Span) -> IResult<Span, Vec<ActionRequest>> {
    separated_nonempty_list(tag("/"), parse_action)(text)
}

fn parse_segment_indicator(text: Span) -> IResult<Span, usize> {
    many1_count(tag("-"))(text)
}

fn parse_segment_header(text: Span) -> IResult<Span, SegmentHeader> {
    let position: Position = text.into();
    let (text, level) = parse_segment_indicator(text)?;
    let (text, opt_request) = opt(parse_action)(text)?;
    if let Some(request) = opt_request{
        Ok((text, SegmentHeader::new_parsed_from_action_request(level, position, &request)))
    }
    else{
        Ok((text, SegmentHeader::new_parsed_minimal(level, position)))
    }
}

fn parse_segment_with_header(text: Span) -> IResult<Span, QuerySegment> {
    let (text, header) = parse_segment_header(text)?;
    let (text, q) = opt(pair(tag("/"), parse_action_path))(text)?;
    if let Some((_, query)) = q{
        Ok((text, QuerySegment::new_from(Some(header), query)))
    }
    else{
        Ok((text, QuerySegment::new_from(Some(header), vec![])))
    }
}
fn parse_segment_without_header(text: Span) -> IResult<Span, QuerySegment> {
    //let (text, _) = opt(tag("/"))(text)?;
    let (text, query) = parse_action_path_nonempty(text)?;
    Ok((text, QuerySegment::new_from(None, query)))
}

fn parse_segment(text: Span) -> IResult<Span, QuerySegment> {
    alt((parse_segment_with_header, parse_segment_without_header))(text)
}

fn parse_query(text: Span) -> IResult<Span, Query> {
    let (text, segments) = separated_list(tag("/"),parse_segment)(text)?;
    Ok((text, Query{segments}))
}


pub fn parse_query_simple(query: &str) -> Result<Vec<ActionRequest>, Error> {
    let (remainder, path) = parse_action_path(Span::new(query)).map_err(|e| Error::General {
        message: format!("Parse error {}", e),
    })?;
    if remainder.fragment().len() > 0 {
        Err(Error::ParseError {
            message: format!("Can't parse query completely: '{}'", remainder.fragment()),
            position: remainder.into(),
        })
    } else {
        Ok(path)
    }
}

pub fn parse(query: &str) -> Result<Query, Error> {
    let (remainder, query) = parse_query(Span::new(query)).map_err(|e| Error::General {
        message: format!("Parse error {}", e),
    })?;
    if remainder.fragment().len() > 0 {
        Err(Error::ParseError {
            message: format!("Can't parse query completely: '{}'", remainder.fragment()),
            position: remainder.into(),
        })
    } else {
        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::ActionParameter;

    #[test]
    fn parse_action_test() -> Result<(), Box<dyn std::error::Error>> {
        let (_remainder, action) = parse_action(Span::new("abc-def"))?;
        assert_eq!(action.name, "abc");
        assert_eq!(action.parameters.len(), 1);
        match &action.parameters[0] {
            ActionParameter::String(txt, _) => assert_eq!(txt, "def"),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn parse_path_test() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, path) = parse_action_path(Span::new("abc-def/xxx-123"))?;
        println!("REMAINDER: {:#?}", remainder);
        println!("PATH:      {:#?}", path);
        assert_eq!(remainder.fragment().len(), 0);
        assert_eq!(remainder.to_string().len(), 0);
        Ok(())
    }
    #[test]
    fn parse_query_test() -> Result<(), Error> {
        let path = parse_query_simple("")?;
        assert_eq!(path.len(), 0);
        let path = parse_query_simple("abc-def");
        println!("{:#?}", path);
        /*        assert_eq!(path.len(),1);
        let path  = parse_query("abc-def/xxx-123")?;
        assert_eq!(path.len(),2);*/
        Ok(())
    }

    #[test]
    fn parse_parameter_entity_test() -> Result<(), Error> {
        let path = parse_query_simple("abc-~~x-~123")?;
        assert_eq!(path.len(), 1);
        if let ActionParameter::String(txt, _pos) = &path[0].parameters[0] {
            assert_eq!(txt, "~x");
        } else {
            assert!(false);
        }
        if let ActionParameter::String(txt, _pos) = &path[0].parameters[1] {
            assert_eq!(txt, "-123");
        } else {
            assert!(false);
        }
        Ok(())
    }

    #[test]
    fn parse_simple_parameter_test() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, param) = parameter(Span::new("abc"))?;
        match &param {
            ActionParameter::String(s, _) => assert_eq!(s, "abc"),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn parse_escaped_parameter_test() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, param) = parameter(Span::new("abc~~~_~0%21"))?;
        match &param {
            ActionParameter::String(s, _) => assert_eq!(s, "abc~--0!"),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn parse_segment_header1() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, sh) = parse_segment_header(Span::new("-"))?;
        assert_eq!(sh.level,1);
        assert_eq!(sh.name, "");
        let (remainder, sh) = parse_segment_header(Span::new("--"))?;
        assert_eq!(sh.level,2);
        Ok(())
    }
    #[test]
    fn parse_segment_header2() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, sh) = parse_segment_header(Span::new("-abc"))?;
        assert_eq!(sh.level,1);
        assert_eq!(sh.name,"abc");
        let (remainder, sh) = parse_segment_header(Span::new("--abc-d-ef"))?;
        assert_eq!(sh.level,2);
        assert_eq!(sh.name,"abc");
        assert_eq!(sh.parameters.len(),2);
        assert_eq!(sh.parameters[0].to_string(),"d");
        assert_eq!(sh.parameters[1].to_string(),"ef");
        Ok(())
    }
    #[test]
    fn parse_segment_without_header1() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, segment) = parse_segment(Span::new("abc-def/xyz"))?;
        assert!(segment.header.is_none());
        assert_eq!(segment.query.len(),2);
        Ok(())
    }
    #[test]
    fn parse_segment1() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, segment) = parse_segment(Span::new("-abc"))?;
        assert_eq!(segment.header.as_ref().unwrap().level,1);
        assert_eq!(segment.header.as_ref().unwrap().name,"abc");
        assert_eq!(segment.query.len(),0);
        Ok(())
    }
    #[test]
    fn parse_segment2() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, segment) = parse_segment(Span::new("-abc/x-y/-next"))?;
        assert_eq!(segment.header.as_ref().unwrap().level,1);
        assert_eq!(segment.header.as_ref().unwrap().name,"abc");
        assert_eq!(segment.query.len(),1);
        assert_eq!(segment.query[0].name,"x");
        Ok(())
    }
    #[test]
    fn parse_empty1() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, query) = parse_query(Span::new(""))?;
        assert_eq!(query.segments.len(),0);
        Ok(())
    }
    #[test]
    fn parse_empty2() -> Result<(), Box<dyn std::error::Error>> {
        let query = parse("")?;
        assert_eq!(query.segments.len(),0);
        Ok(())
    }
    
    #[test]
    fn parse1() -> Result<(), Box<dyn std::error::Error>> {
        let query = parse("-abc/x-y")?;
        assert_eq!(query.segments.len(),1);
        assert_eq!(query.segments[0].header.as_ref().unwrap().name,"abc");
        assert_eq!(query.segments[0].query[0].name,"x");
        Ok(())
    }
    #[test]
    fn parse2() -> Result<(), Box<dyn std::error::Error>> {
        let query = parse("abc-def/-/x-y")?;
        assert_eq!(query.segments.len(),2);
        assert_eq!(query.segments[0].query[0].name,"abc");
        assert_eq!(query.segments[1].query[0].name,"x");
        Ok(())
    }
    
}
