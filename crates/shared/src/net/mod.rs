use bincode::{Decode, Encode};
use crate::AsyncResult;

pub trait Session {
    /// Sends a WoW packet to the client.
    fn send_packet<'a, T: WoWPacket + Send + 'a>(&'a mut self, pkt: T) -> AsyncResult<()>;
}

pub trait WoWPacket: Encode + Decode {
}
