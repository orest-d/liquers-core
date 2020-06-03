#[macro_use]
use nom;

extern crate nom_locate;
use nom_locate::LocatedSpan;


use nom::*;
use nom::bytes::complete::{tag, take_while, take_while1, take_while_m_n};
use nom::character::complete::{digit1};
use nom::multi::{many0, separated_list};
use nom::character::{is_alphanumeric, is_alphabetic, is_hex_digit};
use nom::combinator::{cut};
use nom::branch::{alt};
use nom::sequence::pair;

use percent_encoding::{percent_decode_str, PercentDecode};

use crate::query::{ActionParameter, ActionRequest, Position};
use crate::error::Error;

type Span<'a> = LocatedSpan<&'a str>;

impl<'a> From<Span<'a>> for Position{
    fn from(span:Span<'a>)->Position{
        Position{
            offset:span.location_offset(),
            line:span.location_line(),
            column:span.get_utf8_column()
        }
    }
}

fn identifier(text:Span) ->IResult<Span, String>{
    let (text, a) =take_while1(|c:char| {c.is_alphabetic()||c=='_'})(text)?;
    let (text, b) =take_while(|c:char| {c.is_alphanumeric()||c=='_'})(text)?;

    Ok((text, format!("{}{}",a,b)))
}

fn parameter_text(text:Span) ->IResult<Span, String>{
    let (text, par) =take_while1(|c:char| {c.is_alphanumeric()||c=='_'})(text)?;
    Ok((text, format!("{}",par)))
}

fn percent_encoding(text:Span) ->IResult<Span, String>{
    let (text, _percent) =tag("%")(text)?;
    let (text, hex) = cut(take_while_m_n(2, 2, |c:char| c.is_digit(16)))(text)?;
    Ok((text, format!("%{}",hex)))
}

fn tilde_entity(text:Span) ->IResult<Span, String>{
    let (text, _tilde) = tag("~")(text)?;
    Ok((text, "~".to_owned()))
}

fn minus_entity(text:Span) ->IResult<Span, String>{
    let (text, _tilde) = tag("_")(text)?;
    Ok((text, "-".to_owned()))
}

fn negative_number_entity(text:Span) ->IResult<Span, String>{
    let (text, number) = digit1(text)?;
    Ok((text, format!("-{}",number)))
}

fn parameter_entity(text:Span) ->IResult<Span, String>{
    let (text, _start) = tag("~")(text)?;
    let position:Position = text.into();
    let (text, entity) = cut(alt((
        tilde_entity,
        minus_entity,
        negative_number_entity)))(text)?;
    Ok((text, format!("{}",entity)))
}

fn parameter(text:Span) ->IResult<Span, ActionParameter>{
    let position:Position = text.into();
    let (text, par) =many0(alt((parameter_text, parameter_entity, percent_encoding)))(text)?;
//    let err: nom::Err<(Span, nom::error::ErrorKind)> = nom::error::make_error(text, nom::error::ErrorKind::Escaped);
    let par = par.join("");
    let par = percent_decode_str(&par).decode_utf8().map_err(|e| 
        nom::Err::Failure(nom::error::ParseError::from_error_kind(text, nom::error::ErrorKind::Escaped)))?;

    Ok((text, ActionParameter::new_parsed(par.to_string(), position)))
}


fn parse_action(text:Span) ->IResult<Span, ActionRequest>{
    let position:Position = text.into();
    let (text, name) =identifier(text)?;
    let (text, p) =many0(pair(tag("-"),parameter))(text)?;

    Ok((text, ActionRequest{name:name, position, parameters:p.iter().map(|x| x.1.clone()).collect()}))
}

fn parse_action_path(text:Span) -> IResult<Span, Vec<ActionRequest>>{
    separated_list(tag("/"), parse_action)(text)
}

pub fn parse_query(query:&str)-> Result<Vec<ActionRequest>, Error>{
    let (remainder, path)  = parse_action_path(Span::new(query)).map_err(|e| Error::General{message:format!("Parse error {}",e)})?;
    if remainder.fragment().len()>0{
        Err(Error::ParseError{message:format!("Can't parse query completely: '{}'",remainder.fragment()), position:remainder.into()})
    }
    else{
        Ok(path)
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use crate::query::{ActionParameter};

    #[test]
    fn parse_action_test() -> Result<(), Box<dyn std::error::Error>>{
        let (_remainder, action)  = parse_action(Span::new("abc-def"))?;
        assert_eq!(action.name,"abc");
        assert_eq!(action.parameters.len(),1);
        match &action.parameters[0]{
            ActionParameter::String(txt,_) => assert_eq!(txt, "def"),
            _ => assert!(false)
        }
        Ok(())
    }
    #[test]
    fn parse_path_test() -> Result<(), Box<dyn std::error::Error>>{
        let (remainder, path)  = parse_action_path(Span::new("abc-def/xxx-123"))?;
        println!("REMAINDER: {:#?}",remainder);
        println!("PATH:      {:#?}",path);
        assert_eq!(remainder.fragment().len(),0);
        assert_eq!(remainder.to_string().len(),0);
        Ok(())
    }
    #[test]
    fn parse_query_test() -> Result<(), Error>{
        let path  = parse_query("")?;
        assert_eq!(path.len(),0);
        let path  = parse_query("abc-def");
        println!("{:#?}",path);
/*        assert_eq!(path.len(),1);
        let path  = parse_query("abc-def/xxx-123")?;
        assert_eq!(path.len(),2);*/
        Ok(())
    }

    #[test]
    fn parse_parameter_entity_test() -> Result<(), Error>{
        let path  = parse_query("abc-~~x-~123")?;
        assert_eq!(path.len(),1);
        if let ActionParameter::String(txt,_pos) = &path[0].parameters[0]{
            assert_eq!(txt, "~x");
        }
        else{
            assert!(false);
        }
        if let ActionParameter::String(txt,_pos) = &path[0].parameters[1]{
            assert_eq!(txt, "-123");
        }
        else{
            assert!(false);
        }
        Ok(())
    }

    #[test]
    fn parse_simple_parameter_test() -> Result<(), Box<dyn std::error::Error>>{
        let (remainder,param)  = parameter(Span::new("abc"))?;
        match &param{
            ActionParameter::String(s,_)=>assert_eq!(s,"abc"),
            _ => assert!(false)
        }
        Ok(())
    }
    #[test]
    fn parse_escaped_parameter_test() -> Result<(), Box<dyn std::error::Error>>{
        let (remainder,param)  = parameter(Span::new("abc~~~_~0%21"))?;
        match &param{
            ActionParameter::String(s,_)=>assert_eq!(s,"abc~--0!"),
            _ => assert!(false)
        }
        Ok(())
    }


}