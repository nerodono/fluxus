use bitflags::bitflags;
use integral_enum::integral_enum;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PktFlags: u8 {
        const RESERVED0 = 1 << 0;
        const RESERVED1 = 1 << 1;
        const RESERVED2 = 1 << 2;
    }
}

#[rustfmt::skip]
#[integral_enum(u8)]
pub enum PktType {
    Error        = 0x00,
    ReqInfo      = 0x01,
    Connected    = 0x02,
    Disconnected = 0x03,

    Authenticate = 0x04,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PktBase {
    pub type_: PktType,
    pub flags: PktFlags,
}

impl PktBase {
    pub fn try_decode(int: u8) -> Option<Self> {
        PktFlags::from_bits(int & 0b111).and_then(|reinterp| {
            Some(PktBase {
                type_: PktType::try_from(int >> 3).ok()?,
                flags: reinterp,
            })
        })
    }
}
