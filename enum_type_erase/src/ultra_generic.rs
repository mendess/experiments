use super::*;

impl TryFrom<&XmlElement> for DirLight {
    type Error = ParseErr;
    fn try_from(_elem: &XmlElement) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<&XmlElement> for SpotLight {
    type Error = ParseErr;
    fn try_from(_elem: &XmlElement) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<&XmlElement> for LightPoint {
    type Error = ParseErr;
    fn try_from(_elem: &XmlElement) -> Result<Self, Self::Error> {
        todo!()
    }
}

fn upcast<F, A, O>(arg: A) -> Result<O, F::Error>
where
    F: TryFrom<A>,
    F: Into<O>,
{
    F::try_from(arg).map(Into::into)
}

pub type ParseFn = fn(&XmlElement) -> Result<Light, ParseErr>;

pub static PARSE_TABLE: [ParseFn; 3] = [
    |a| upcast::<LightPoint, _, _>(a),
    |a| upcast::<DirLight, _, _>(a),
    |a| upcast::<SpotLight, _, _>(a),
];
