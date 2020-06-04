use serde_json;

use std::result::Result;

use crate::error::Error;
use std::convert::{TryFrom, TryInto};
use strum::IntoEnumIterator;
use strum_macros::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value{
    None,
    Text(String),
    Integer(i32),
    Real(f64),
    Bool(bool)
}

#[derive(EnumIter, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ValueSerializationFormats{
    Text,
    Json,
    SerdeJson
}

pub fn media_type_from_extension(extension:&str)->&'static str{
    match extension{
        "json"=>"application/json",
        "js"=>"text/javascript",
        "txt"=>"text/plain",
        "html"=>"text/html",
        "htm"=>"text/html",
        "md"=>"text/markdown",
        "xls"=>"application/vnd.ms-excel",
        "xlsx"=>"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ods"=>"application/vnd.oasis.opendocument.spreadsheet",
        "tsv"=>"text/tab-separated-values",
        "csv"=>"text/csv",
        "msgpack"=>"application/x-msgpack",
        "hdf5"=>"application/x-hdf",
        "h5"=>"application/x-hdf",
        "png"=>"image/png",
        "svg"=>"image/svg+xml",
        "jpg"=>"image/jpeg",
        "jpeg"=>"image/jpeg",
        "b"=>"application/octet-stream",
        "pkl"=>"application/octet-stream",
        "pickle"=>"application/octet-stream",
        "wasm"=>"application/wasm",
        _ => "application/octet-stream"
    }
}

trait SerializationFormats where Self:Sized + IntoEnumIterator + std::fmt::Debug + std::cmp::PartialEq {
    fn from_name(name:&str)->Option<Self>{
        for x in Self::iter(){
            if x.to_name()==name{
                return Some(x)
            }
        }
        None
    }
    fn to_name(&self)->String{
        format!("{:?}",self)
    }
    fn media_type(&self)->&'static str{
        self.default_extension()
        .split('.')
        .last()
        .map(|x| media_type_from_extension(x))
        .unwrap_or("application/octet-stream")
    }

    fn default_extension(&self)->&'static str{
        for ext in Self::supported_extensions(){
            if let Some(fmt) = Self::from_extension(ext){
                if fmt == *self{
                    return ext;
                }
            }
        }
        ""
    }
    fn supported_extensions()->&'static [&'static str];
    fn from_extension(ext:&str)->Option<Self>;
    fn extension_from_filename(filename:&str)->Option<&'static str>{
        Self::supported_extensions().iter()
        .enumerate()
        .filter(|(i,x)| filename.ends_with(*x))
        .map(|(i,x)| (x.len(),i))
        .max()
        .map(|(_,i)| Self::supported_extensions()[i])
    }
    fn from_filename(filename:&str)->Option<Self>{
        Self::extension_from_filename(filename).and_then(|x| Self::from_extension(x))
    }
}

impl SerializationFormats for ValueSerializationFormats{
    fn supported_extensions()->&'static [&'static str]{
        &["txt", "json", "serde.json"]
    }
    fn from_extension(ext:&str)->Option<Self>{
        match ext{
            "txt" => Some(Self::Text),
            "json" => Some(Self::Json),
            "serde.json" => Some(Self::SerdeJson),
            _ => None
        }
    }
}

trait ValueSerializer where Self:Sized{
    type Formats:SerializationFormats;
    fn type_identifier(&self)->String;
    fn default_extension(&self)->String;
    fn default_media_type(&self)->String;
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>;
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>;
}

impl ValueSerializer for Value{
    type Formats = ValueSerializationFormats;
    fn type_identifier(&self)->String{
        match self {
            Value::None => String::from("none"),
            Value::Text(_) => String::from("text"),
            Value::Integer(_) => String::from("int"),
            Value::Real(_) => String::from("real"),
            Value::Bool(_) => String::from("bool"),
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
            Value::Bool(_) => Err(Error::ConversionError{message:format!("Can't convert Bool to integer")}),
            Value::Integer(x) => Ok(x),
            Value::Real(_) => Err(Error::ConversionError{message:format!("Can't convert real number to integer")}),
        }
    }
}

impl From<i32> for Value{
    fn from(value: i32) -> Value{
        Value::Integer(value)
    }
}

impl TryFrom<Value> for f64{
    type Error=Error;
    fn try_from(value: Value) -> Result<Self, Self::Error>{
        match value{
            Value::None => Err(Error::ConversionError{message:format!("Can't convert None to real number")}),
            Value::Text(_) => Err(Error::ConversionError{message:format!("Can't convert Text to real number")}),
            Value::Bool(_) => Err(Error::ConversionError{message:format!("Can't convert Bool to real number")}),
            Value::Integer(x) => Ok(x as f64),
            Value::Real(x) => Ok(x),
        }
    }
}

impl From<f64> for Value{
    fn from(value: f64) -> Value{
        Value::Real(value)
    }
}

impl TryFrom<Value> for bool{
    type Error=Error;
    fn try_from(value: Value) -> Result<Self, Self::Error>{
        match value{
            Value::None => Ok(false),
            Value::Text(x) => {
                match &x.to_lowercase()[..]{
                    "true" => Ok(true),
                    "false" => Ok(false),
                    _ => Err(Error::ConversionError{message:format!("Can't convert Text {} to bool",x)})
                }
            },
            Value::Bool(x) => Ok(x),
            Value::Integer(x) => Ok(x!=0),
            Value::Real(x) => Ok(x!=0.0),
        }
    }
}

impl From<bool> for Value{
    fn from(value: bool) -> Value{
        Value::Bool(value)
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
            Value::Bool(x) => Ok(format!("{}",x)),
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
        let x:f64 = v.try_into()?;
        assert_eq!(x,123.1);
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
    #[test]
    fn test_convert_bool() -> Result<(), Box<dyn std::error::Error>>{
        let v = Value::from(true);
        assert_eq!(v,Value::Bool(true));
        let v = Value::from(false);
        assert_eq!(v,Value::Bool(false));
        Ok(())
    }   
}