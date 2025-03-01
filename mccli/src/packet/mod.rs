pub mod types;

use std::{
    borrow::Cow,
    io::{self, BufWriter, Cursor, Read, Write},
};
use types::{McType, String, VarInt};

pub struct Packet<'p> {
    packet_id: VarInt,
    payload: Cow<'p, [u8]>,
}

impl Packet<'static> {
    pub fn handshake(version: u16) -> Self {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        tracing::trace!("creating handshake packet");
        VarInt::from(version).write(&mut cursor).unwrap();
        String::borrowed("localhost").write(&mut cursor).unwrap();
        25565u16.write(&mut cursor).unwrap();
        VarInt::from(1 /*status*/).write(&mut cursor).unwrap();
        Self {
            packet_id: VarInt::from(0x00),
            payload: buffer.into(),
        }
    }

    pub fn status_request() -> Self {
        Self {
            packet_id: 0x00.into(),
            payload: Default::default(),
        }
    }
}

impl Packet<'_> {
    pub fn read<R: Read>(mut r: R) -> io::Result<Self> {
        tracing::info!("reading length");
        let length: usize = VarInt::read(&mut r)?.try_into().map_err(io::Error::other)?;
        tracing::info!(%length, "reading packet id");
        let packet_id = VarInt::read(&mut r)?;
        tracing::info!(%length, ?packet_id, "reading payload");
        let mut buffer = vec![0; length - packet_id.len()];
        r.read_exact(&mut buffer[0..length - packet_id.len()])?;
        Ok(Self {
            packet_id,
            payload: Cow::Owned(buffer),
        })
    }

    #[tracing::instrument(skip_all, fields(self.packet_id = ?self.packet_id))]
    pub fn write<W: Write>(&self, w: W) -> io::Result<()> {
        let mut w = BufWriter::new(w);
        let length = VarInt::try_from(self.packet_id.len() + self.payload.len())
            .map_err(io::Error::other)?;
        tracing::trace!(?length, "writing length");
        length.write(&mut w)?;
        tracing::trace!("writing packet id");
        self.packet_id.write(&mut w)?;
        tracing::trace!(payload.len = self.payload.len(), "writing payload");
        w.write_all(&self.payload)?;
        Ok(())
    }

    pub fn reader(&self) -> PacketReader<'_> {
        PacketReader {
            packet: self,
            position: 0,
        }
    }
}

pub struct PacketReader<'p> {
    packet: &'p Packet<'p>,
    position: usize,
}

impl<'t> PacketReader<'t> {
    pub fn next<T: McType + 't>(&mut self) -> io::Result<T> {
        let mut cursor = Cursor::new(&self.packet.payload[self.position..]);
        let t = T::read(&mut cursor)?;
        self.position += cursor.position() as usize;
        Ok(t)
    }
}
