use derive_more::From;

#[derive(From)]
pub enum Light {
    Point(LightPoint),
    Dir(DirLight),
    Spot(SpotLight),
}

// dummys for context
pub struct XmlElement;
pub struct ParseErr;
pub struct DirLight;
pub struct SpotLight;
pub struct LightPoint;
