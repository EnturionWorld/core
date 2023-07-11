use bincode::de::Decoder;
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{impl_borrow_decode, Decode, Encode};
use kitros_derive::wow_auth_packet;

#[derive(Copy, Clone)]
#[repr(u8)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum AuthCommand {
    AuthLogonChallenge = 0x00,
    AuthLogonProof = 0x01,
    AuthReconnectChallenge = 0x02,
    AuthReconnectProof = 0x03,
    RealmList = 0x10,
    XferInitiate = 0x30,
    XferData = 0x31,
    XferAccept = 0x32,
    XferResume = 0x33,
    XferCancel = 0x34,
}

#[derive(Copy, Clone)]
#[repr(u8)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum AuthResult {
    WowSuccess = 0x00,
    WowFailBanned = 0x03,
    WowFailUnknownAccount = 0x04,
    WowFailIncorrectPassword = 0x05,
    WowFailAlreadyOnline = 0x06,
    WowFailNoTime = 0x07,
    WowFailDbBusy = 0x08,
    WowFailVersionInvalid = 0x09,
    WowFailVersionUpdate = 0x0A,
    WowFailInvalidServer = 0x0B,
    WowFailSuspended = 0x0C,
    WowFailFailNoaccess = 0x0D,
    WowSuccessSurvey = 0x0E,
    WowFailParentcontrol = 0x0F,
    WowFailLockedEnforced = 0x10,
    WowFailTrialEnded = 0x11,
    WowFailUseBattlenet = 0x12,
    WowFailAntiIndulgence = 0x13,
    WowFailExpired = 0x14,
    WowFailNoGameAccount = 0x15,
    WowFailChargeback = 0x16,
    WowFailInternetGameRoomWithoutBnet = 0x17,
    WowFailGameAccountLocked = 0x18,
    WowFailUnlockableLock = 0x19,
    WowFailConversionRequired = 0x20,
    WowFailDisconnected = 0xFF,
}

impl Encode for AuthResult {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let repr = *self as u8;
        ::bincode::Encode::encode(&repr, encoder)
    }
}

impl_borrow_decode!(AuthResult);
impl Decode for AuthResult {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let repr: u8 = ::bincode::Decode::decode(decoder)?;
        Ok(unsafe { std::mem::transmute(repr) })
    }
}

impl Encode for AuthCommand {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let repr = *self as u8;
        ::bincode::Encode::encode(&repr, encoder)
    }
}

impl_borrow_decode!(AuthCommand);
impl Decode for AuthCommand {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let repr: u8 = ::bincode::Decode::decode(decoder)?;
        Ok(unsafe { std::mem::transmute(repr) })
    }
}

#[wow_auth_packet]
pub struct LogonChallengeErrorResponse {
    command: AuthCommand,
    padding: u8,
    auth_result: AuthResult,
}
