use std::error;
use std::fmt;
use crate::query::Position;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error{
    ArgumentNotSpecified,
    ActionNotRegistered{message:String},
    ParseError{message:String, position:Position},
    ParameterError{message:String, position:Position},
    ConversionError{message:String},
    SerializationError{message:String, format:String},
    General{message:String}
}

impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ArgumentNotSpecified => write!(f, "Argument not specified"),
            Error::ActionNotRegistered{message} => write!(f, "Error: {}", message),
            Error::ParseError{message, position} => write!(f, "Error: {} {}", message, position),
            Error::ParameterError{message, position} => write!(f, "Error: {} {}", message, position),
            Error::ConversionError{message} => write!(f, "Error: {}", message),
            Error::SerializationError{message, format:_} => write!(f, "Error: {}", message),
            Error::General{message} => write!(f, "Error: {}", message),
        }
    }    
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            _ => None,
        }
    }
}
