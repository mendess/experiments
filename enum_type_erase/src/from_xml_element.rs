use crate::types::{DirLight, Light, LightPoint, ParseErr, SpotLight, XmlElement};

pub trait FromXmlElement: Sized {
    // not needed if I hardcoded the error type, but I felt like it
    type Error;
    fn parse(elem: &XmlElement) -> Result<Self, Self::Error>;
}

impl FromXmlElement for DirLight {
    type Error = ParseErr;
    fn parse(_elem: &XmlElement) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl FromXmlElement for SpotLight {
    type Error = ParseErr;
    fn parse(_elem: &XmlElement) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl FromXmlElement for LightPoint {
    type Error = ParseErr;
    fn parse(_elem: &XmlElement) -> Result<Self, Self::Error> {
        todo!()
    }
}

fn upcast_parser<P, R>(elem: &XmlElement) -> Result<R, P::Error>
where
    P: FromXmlElement + Into<R>,
{
    P::parse(elem).map(Into::into)
}

pub type ParseFn = fn(&XmlElement) -> Result<Light, ParseErr>;

pub static PARSE_TABLE: [ParseFn; 3] = [
    upcast_parser::<LightPoint, _>,
    upcast_parser::<DirLight, _>,
    upcast_parser::<SpotLight, _>,
];
