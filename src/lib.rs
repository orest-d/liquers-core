#[macro_use]
extern crate nom;
extern crate regex;

extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate percent_encoding;
extern crate strum;
#[macro_use]
extern crate strum_macros;

#[macro_use]
extern crate serde_derive;

pub mod value;
pub mod error;
pub mod query;
pub mod parse;
pub mod action_registry;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
