use bitflags::bitflags;
use integral_enum::integral_enum;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PktFlags: u8 {
        const FLAG0     = 1 << 0;
        const FLAG1     = 1 << 1;
        const FLAG2     = 1 << 2;
    }
}

#[rustfmt::skip]
#[integral_enum(u8)]
pub enum PktType {
    Error        = 0x00,
    ReqInfo      = 0x01,
    Connected    = 0x02,
    Disconnect   = 0x03,

    Authenticate = 0x04,
    UpdateRights = 0x05,

    CreateTcp    = 0x0F,
    CreateHttp   = 0x10,
}

/// Describes base header for all master packets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PktBase {
    pub type_: PktType,
    pub flags: PktFlags,
}

impl PktBase {
    /// Encodes packet as the `u8` that can be sent through
    /// the network.
    ///
    /// ```rust
    /// use tcp_flux::types::pkt_base::{
    ///     PktBase,
    ///     PktFlags,
    ///     PktType,
    /// };
    ///
    /// assert_eq!(PktBase::simple(PktType::Error).encode(), 0);
    /// assert_eq!(
    ///     PktBase::new(PktType::Error, PktFlags::FLAG0).encode(),
    ///     PktFlags::FLAG0.bits()
    /// );
    /// ```
    pub const fn encode(self) -> u8 {
        ((self.type_ as u8) << 3) | self.flags.bits()
    }
}

impl PktBase {
    /// Creates simple [`PktBase`]. Equivalent to passing
    /// empty `PktFlags` in the [`PktBase::new`]
    ///
    /// ```rust
    /// use tcp_flux::types::pkt_base::{
    ///     PktBase,
    ///     PktFlags,
    ///     PktType,
    /// };
    ///
    /// assert_eq!(
    ///     PktBase::simple(PktType::Error),
    ///     PktBase::new(PktType::Error, PktFlags::empty())
    /// );
    /// ```
    pub const fn simple(type_: PktType) -> Self {
        Self::new(type_, PktFlags::empty())
    }

    pub const fn new(type_: PktType, flags: PktFlags) -> Self {
        Self { type_, flags }
    }

    pub fn try_decode(int: u8) -> Option<Self> {
        if let Some(reinterp) = PktFlags::from_bits(int & 0b111) {
            Some(Self::new(PktType::try_from(int >> 3).ok()?, reinterp))
        } else {
            None
        }
    }
}
