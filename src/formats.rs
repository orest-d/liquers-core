use std::result::Result;
use crate::error::Error;

use strum::IntoEnumIterator;
use strum_macros::*;


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

pub trait SerializationFormats where Self:Sized + IntoEnumIterator + std::fmt::Debug + std::cmp::PartialEq {
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

pub trait ValueSerializer where Self:Sized{
    type Formats:SerializationFormats;
    fn type_identifier(&self)->String;
    fn default_extension(&self)->String;
    fn default_media_type(&self)->String;
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>;
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>;
}
