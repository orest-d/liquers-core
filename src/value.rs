use serde_json;

use std::result::Result;

use crate::error::Error;
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value{
    None,
    Text(String),
    Integer(i32),
    Real(f64)
}

trait ValueSerializer where Self:Sized{
    fn identifier(&self)->String;
    fn default_extension(&self)->String;
    fn default_media_type(&self)->String;
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>;
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>;
}

impl ValueSerializer for Value{
    fn identifier(&self)->String{
        match self {
            Value::None => String::from("none"),
            Value::Text(_) => String::from("text"),
            Value::Integer(_) => String::from("int"),
            Value::Real(_) => String::from("real"),
        }
    }
    fn default_extension(&self)->String{
        String::from("json")
    }
    fn default_media_type(&self)->String{
        String::from("application/json")
    }
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>{
        match format{
            "json" => serde_json::to_vec(self).map_err(|e| Error::SerializationError{message:format!("JSON errror {}",e), format:format.to_owned()}),
            _ => Err(Error::SerializationError{message:format!("Unsupported format {}",format), format:format.to_owned()})
        }
    }
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>{
        match format{
            "json" => serde_json::from_slice(b).map_err(|e| Error::SerializationError{message:format!("JSON errror {}",e), format:format.to_owned()}),
            _ => Err(Error::SerializationError{message:format!("Unsupported format {}",format), format:format.to_owned()})
        }
    }
}

impl TryFrom<Value> for i32{
    type Error=Error;
    fn try_from(value: Value) -> Result<Self, Self::Error>{
        match value{
            Value::None => Err(Error::ConversionError{message:format!("Can't convert None to integer")}),
            Value::Text(_) => Err(Error::ConversionError{message:format!("Can't convert Text to integer")}),
            Value::Integer(x) => Ok(x),
            Value::Real(x) => Ok(x as i32),           
        }
    }
}

impl From<i32> for Value{
    fn from(value: i32) -> Value{
        Value::Integer(value)
    }
}

impl TryFrom<Value> for String{
    type Error=Error;
    fn try_from(value: Value) -> Result<Self, Self::Error>{
        match value{
            Value::None => Err(Error::ConversionError{message:format!("Can't convert None to string")}),
            Value::Text(x) => Ok(x),
            Value::Integer(x) => Ok(format!("{}",x)),
            Value::Real(x) => Ok(format!("{}",x)),           
        }
    }
}

impl From<String> for Value{
    fn from(value: String) -> Value{
        Value::Text(value)
    }
}
impl From<&str> for Value{
    fn from(value: &str) -> Value{
        Value::Text(value.to_owned())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test1() -> Result<(), Box<dyn std::error::Error>>{
        println!("Hello.");
        let v = Value::Integer(123);
        let b = v.as_bytes("json")?;
        println!("Serialized    {:?}: {}", v, std::str::from_utf8(&b)?);
        let w:Value = ValueSerializer::from_bytes(&b, "json")?;
        println!("De-Serialized {:?}", w);
        Ok(())
    }   
    #[test]
    fn test_convert_int() -> Result<(), Box<dyn std::error::Error>>{
        let v = Value::Integer(123);
        let x:i32 = v.try_into()?;
        assert_eq!(x,123);
        Ok(())
    }   
    #[test]
    fn test_convert_real() -> Result<(), Box<dyn std::error::Error>>{
        let v = Value::Real(123.1);
        let x:i32 = v.try_into()?;
        assert_eq!(x,123);
        Ok(())
    }   
    #[test]
    fn test_convert_text() -> Result<(), Box<dyn std::error::Error>>{
        let v = Value::from("abc");
        assert_eq!(v,Value::Text("abc".to_owned()));
        let x:String = v.try_into()?;
        assert_eq!(x,"abc");
        Ok(())
    }   
}