use std::{
    borrow::Cow,
    fmt,
    io::{self, Read, Write},
    ops::Deref,
};

pub trait McType {
    fn read<R: Read>(r: R) -> io::Result<Self>
    where
        Self: Sized;
    fn write<W: Write>(&self, w: W) -> io::Result<()>;
}

macro_rules! VarNum {
    ($name:ident: $int:ty | $unsigned:ty) => {
        const _: () = assert!(std::mem::size_of::<$int>() == std::mem::size_of::<$unsigned>());
        const _: () = assert!(<$int>::MIN != 0);
        const _: () = assert!(<$unsigned>::MIN == 0);

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
        pub struct $name {
            int: $int,
        }

        impl $name {
            const SEGMENT_BITS: u8 = 0x7f;
            const CONTINUE_BIT: u8 = 0x80;

            pub fn len(&self) -> usize {
                if self.int < 0 {
                    [5, 10][usize::from(std::mem::size_of::<$int>() != 4)]
                } else if self.int == 0 {
                    1
                } else {
                    match dbg!(<$int>::BITS - self.int.leading_zeros()) {
                        ..8 => 1,
                        8..15 => 2,
                        15..22 => 3,
                        22..29 => 4,
                        29..36 => 5,
                        36..43 => 6,
                        43..50 => 7,
                        50..57 => 8,
                        57..64 => 9,
                        64.. => 10,
                    }
                }
            }
        }

        impl McType for $name {
            fn read<R: Read>(mut r: R) -> io::Result<Self> {
                let mut len = 0_u8;
                let mut current_byte = 0u8;
                let mut int = 0;
                for i in 0..128 {
                    tracing::debug!("reading byte {i}");
                    r.read_exact(std::slice::from_mut(&mut current_byte))?;

                    int |= <$int>::from(current_byte & Self::SEGMENT_BITS) << len;

                    if current_byte & Self::CONTINUE_BIT == 0 {
                        return Ok(Self { int });
                    };

                    len += 7;

                    if u32::from(len) >= <$int>::BITS {
                        return Err(io::ErrorKind::InvalidData.into());
                    };
                }
                panic!("infinite loop when reading {}", stringify!($name));
            }

            #[tracing::instrument(fields(?self), skip_all)]
            fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
                let mut int = self.int;
                for _ in 0..128 {
                    if int & !<$int>::from(Self::SEGMENT_BITS) == 0 {
                        let byte = int as u8;
                        w.write_all(std::slice::from_ref(&byte))?;
                        tracing::trace!(?byte, "wrote one byte 0x{byte:x}");
                        return Ok(());
                    }

                    let byte = (int as u8) & Self::SEGMENT_BITS | Self::CONTINUE_BIT;
                    w.write_all(std::slice::from_ref(&byte))?;
                    tracing::trace!(?byte, "wrote one byte 0x{byte:x}");

                    int = ((int as $unsigned) >> 7) as $int;
                }
                panic!("infinite loop when writing {}", stringify!($name));
            }
        }

        impl From<u8> for $name {
            fn from(v: u8) -> Self {
                Self {
                    int: <$int>::from(v),
                }
            }
        }

        impl From<i8> for $name {
            fn from(v: i8) -> Self {
                Self {
                    int: <$int>::from(v),
                }
            }
        }

        impl From<u16> for $name {
            fn from(v: u16) -> Self {
                Self {
                    int: <$int>::from(v),
                }
            }
        }

        impl From<i16> for $name {
            fn from(v: i16) -> Self {
                Self {
                    int: <$int>::from(v),
                }
            }
        }

        impl From<i32> for $name {
            fn from(v: i32) -> Self {
                Self {
                    int: <$int>::from(v),
                }
            }
        }

        impl TryFrom<u64> for $name {
            type Error = <$int as TryFrom<u64>>::Error;

            fn try_from(v: u64) -> Result<Self, Self::Error> {
                Ok(Self { int: v.try_into()? })
            }
        }

        impl TryFrom<usize> for $name {
            type Error = <$int as TryFrom<usize>>::Error;

            fn try_from(v: usize) -> Result<Self, Self::Error> {
                Ok(Self { int: v.try_into()? })
            }
        }

        impl TryFrom<$name> for usize {
            type Error = <usize as TryFrom<$int>>::Error;

            fn try_from(this: $name) -> Result<Self, Self::Error> {
                this.int.try_into()
            }
        }
    };
}

VarNum!(VarInt: i32 | u32);

impl TryFrom<u32> for VarInt {
    type Error = <i32 as TryFrom<u32>>::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self {
            int: i32::try_from(value)?,
        })
    }
}

impl TryFrom<i64> for VarInt {
    type Error = <i32 as TryFrom<i64>>::Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            int: i32::try_from(value)?,
        })
    }
}

VarNum!(VarLong: i64 | u64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct String<'s>(Cow<'s, str>);

impl fmt::Display for String<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'s> String<'s> {
    pub fn borrowed(s: &'s str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl Deref for String<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl McType for String<'_> {
    fn read<R: Read>(mut r: R) -> io::Result<String<'static>> {
        let length = VarInt::read(&mut r)?.try_into().map_err(io::Error::other)?;
        let mut buffer = vec![0; length];
        r.read_exact(&mut buffer)?;
        let s = std::string::String::from_utf8(buffer).map_err(io::Error::other)?;

        Ok(String(Cow::Owned(s)))
    }

    fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
        VarInt {
            int: self.0.len().try_into().map_err(io::Error::other)?,
        }
        .write(&mut w)?;
        w.write_all(self.0.as_bytes())?;
        Ok(())
    }
}

pub mod server {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Status {
        pub version: Version,
        #[serde(rename = "enforcesSecureChat", default)]
        pub enforces_secure_chat: bool,
        pub description: Description,
        pub players: Players,
        pub favicon: Option<String>,
        pub modinfo: Option<ModInfo>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Version {
        pub name: String,
        pub protocol: u16,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Players {
        pub max: u64,
        pub online: u64,
        #[serde(default)]
        pub sample: Vec<Player>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Player {
        pub id: String,
        pub name: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    pub enum Description {
        Text(std::string::String),
        Colored(ColoredText),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ColoredText {
        #[serde(default)]
        pub bold: bool,
        pub color: Option<String>,
        #[serde(default)]
        pub extra: Vec<ColoredText>,
        pub text: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ModInfo {
        #[serde(rename = "modList")]
        pub mod_list: Vec<String>,
        pub r#type: String,
    }
}

macro_rules! num {
    ($($int:ty),*$(,)?) => {
        $(
        impl McType for $int {
            fn read<R: Read>(mut r: R) -> io::Result<Self>
            where
                Self: Sized + 'static,
            {
                let mut buffer = [0u8; std::mem::size_of::<Self>()];
                r.read_exact(&mut buffer)?;
                Ok(Self::from_be_bytes(buffer))
            }

            fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
                w.write_all(&self.to_be_bytes())
            }
        }
        )*
    };
}

num!(u8, i8, u16, i16, u32, i32, u64, i64);

#[cfg(test)]
mod test {
    use super::*;
    use proptest::proptest;
    use std::io::Cursor;

    proptest! {
        #[test]
        fn test_var_int(int in i32::MIN..i32::MAX) {
            let var_int = VarInt { int };
            let mut buffer = Vec::<u8>::new();
            let mut cursor = Cursor::new(&mut buffer);

            var_int.write(&mut cursor).unwrap();
            let var_int2 = VarInt::read(Cursor::new(&buffer)).unwrap();
            assert_eq!(var_int, var_int2);

            assert_eq!(var_int.len(), buffer.len());
        }

        #[test]
        fn test_var_long(int in i64::MIN..i64::MAX) {
            let var_long = VarLong { int };
            let mut buffer = Vec::<u8>::new();
            let mut cursor = Cursor::new(&mut buffer);

            var_long.write(&mut cursor).unwrap();
            let var_long2 = VarLong::read(Cursor::new(&buffer)).unwrap();
            assert_eq!(var_long, var_long2);

            assert_eq!(var_long.len(), buffer.len());
        }
    }

    #[test]
    fn var_int_examples() {
        for (int, bytes) in [
            (0, &[0x00u8][..]),
            (1, &[0x01]),
            (2, &[0x02]),
            (127, &[0x7f]),
            (128, &[0x80, 0x01]),
            (255, &[0xff, 0x01]),
            (25565, &[0xdd, 0xc7, 0x01]),
            (2097151, &[0xff, 0xff, 0x7f]),
            (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
        ] {
            eprintln!("testing {int}");
            let mut buffer = Vec::new();
            let cursor = Cursor::new(&mut buffer);
            VarInt { int }.write(cursor).unwrap();
            assert_eq!(bytes, buffer, "{int} was not encoded correctly");

            let cursor = Cursor::new(&bytes);
            let read_int = VarInt::read(cursor).unwrap();
            assert_eq!(int, read_int.int, "{int} was not decoded correctly");

            assert_eq!(VarInt { int }.len(), bytes.len());
        }
    }

    #[test]
    fn var_long_examples() {
        for (int, bytes) in [
            (0, &[0x00u8][..]),
            (1, &[0x01]),
            (2, &[0x02]),
            (127, &[0x7f]),
            (128, &[0x80, 0x01]),
            (255, &[0xff, 0x01]),
            (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
            (
                9223372036854775807,
                &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            ),
            (
                -1,
                &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -2147483648,
                &[0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -9223372036854775808,
                &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            ),
        ] {
            eprintln!("testing {int}");
            let mut buffer = Vec::new();
            let cursor = Cursor::new(&mut buffer);
            VarLong { int }.write(cursor).unwrap();
            assert_eq!(bytes, buffer, "{int} was not encoded correctly");

            let cursor = Cursor::new(&bytes);
            let read_int = VarLong::read(cursor).unwrap();
            assert_eq!(int, read_int.int, "{int} was not decoded correctly");

            assert_eq!(VarLong { int }.len(), bytes.len());
        }
    }
}
