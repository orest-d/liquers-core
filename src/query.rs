use std::result::Result;
use std::fmt::Display;
use crate::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position{
    pub offset:usize,
    pub line:u32,
    pub column:usize
}

impl Position{
    pub fn unknown()->Position{
        Position{offset:0, line:0, column:0}
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line==0{
            write!(f, "(unknown position)")
        }
        else if self.line>1{
            write!(f, "line {}, position {}", self.line, self.column)
        }
        else{
            write!(f, "position {}", self.column)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionParameter{
    String(String, Position),
    Link(String, Position)
}

impl ActionParameter{
    pub fn new(parameter:&str)->ActionParameter{
        ActionParameter::String(parameter.to_owned(), Position::unknown())
    }
    pub fn new_parsed(parameter:String, position:Position)->ActionParameter{
        ActionParameter::String(parameter, position)
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionRequest{
    pub name:String,
    pub position:Position,
    pub parameters: Vec<ActionParameter>
}

#[derive(Debug)]
pub struct ActionParametersSlice<'a>(pub &'a [ActionParameter]);

pub trait Environment<T>{
    fn eval(&mut self, input:T, query:&str)->Result<T,Error>;
}

pub trait TryActionParametersInto<T,E>{
    fn try_parameters_into(&mut self, env:&mut E)->Result<T,Error>;
}

pub trait TryParameterFrom where Self: std::marker::Sized{
    fn try_parameter_from(text:&str)->Result<Self,String>;
}

impl TryParameterFrom for i32{
    fn try_parameter_from(text:&str)->Result<Self,String>{
       text.parse().map_err(|_| format!("Can't parse '{}' as integer",text))
    }
}

impl TryParameterFrom for String{
    fn try_parameter_from(text:&str)->Result<Self,String>{
        Ok(text.to_owned())
    }
}

impl<'a,T,E> TryActionParametersInto<T,E>  for ActionParametersSlice<'a> where T:TryParameterFrom{
    fn try_parameters_into(&mut self, env:&mut E)->Result<T,Error>{
        if self.0.is_empty(){
            Err(Error::ArgumentNotSpecified)
        }
        else{
            match &self.0[0]{
                ActionParameter::String(x, position)=>{
                    let v:T = T::try_parameter_from(&x).map_err(|message| Error::ParameterError{message, position:position.clone()})?;
                    self.0=&self.0[1..];
                    Ok(v)
                },
                _ =>{
                    Err(Error::General{message:"Not implemented".to_owned()})
                }
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn parameters_into_i32() -> Result<(), Box<dyn std::error::Error>>{
        let v = [ActionParameter::new("123"),ActionParameter::new("234")];
        let mut par = ActionParametersSlice(&v[..]);
        let x:i32=par.try_parameters_into(&mut ())?;
        assert_eq!(x,123);
        let x:i32=par.try_parameters_into(&mut ())?;
        assert_eq!(x,234);
        Ok(())
    }
    #[test]
    fn parameters_into_str() -> Result<(), Box<dyn std::error::Error>>{
        let v = [ActionParameter::new("123"),ActionParameter::new("234")];
        let mut par = ActionParametersSlice(&v[..]);
        let x:String=par.try_parameters_into(&mut ())?;
        assert_eq!(x,"123");
        let x:i32=par.try_parameters_into(&mut ())?;
        assert_eq!(x,234);
        Ok(())
    }
}