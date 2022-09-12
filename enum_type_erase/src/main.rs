mod from_xml_element;
pub mod types;
mod ultra_generic;

pub use types::*;

fn main() {
    let _: Result<Light, ParseErr> = from_xml_element::PARSE_TABLE[0](&XmlElement);
    let _: Result<Light, ParseErr> = ultra_generic::PARSE_TABLE[0](&XmlElement);
}
