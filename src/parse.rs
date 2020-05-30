use nom;

extern crate nom_locate;
use nom_locate::LocatedSpan;


use nom::bytes::complete::{tag, take_while, take_while1};
use nom::*;
use nom::multi::{many0, separated_list};
use nom::character::{is_alphanumeric, is_alphabetic};
use nom::sequence::pair;

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
    let (text, a) =take_while1(|c| {is_alphabetic(c as u8)||c=='_'})(text)?;
    let (text, b) =take_while(|c| {is_alphanumeric(c as u8)||c=='_'})(text)?;

    Ok((text, format!("{}{}",a,b)))
}
fn parameter(text:Span) ->IResult<Span, ActionParameter>{
    let position:Position = text.into();
    let (text, par) =take_while(|c| {c!='-'&&c!='/'})(text)?;

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
        let path  = parse_query("abc-def")?;
        assert_eq!(path.len(),1);
        let path  = parse_query("abc-def/xxx-123")?;
        assert_eq!(path.len(),2);
        Ok(())
    }

}