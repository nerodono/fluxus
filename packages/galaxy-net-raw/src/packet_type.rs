use std::fmt::Display;

use bitflags::bitflags;
use integral_enum::IntegralEnum;

bitflags! {
    /// Flags for the packet type.
    pub struct PacketFlags: u8 {
        const COMPRESSED = 1 << 0;
        const SHORT      = 1 << 1;
        const SHORT_C    = 1 << 2;
    }
}

#[derive(IntegralEnum)]
#[repr(u8)]
pub enum PacketType {
    Failure = 0,
    Ping = 1,

    Connect = 2,
    Forward = 3,
    Disconnect = 4,

    Authorize = 5,
    Server = 6,

    UpdateRights = 7,

    Extended = 31,
}

impl Display for PacketType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl PacketType {
    /// Encodes packet type to the `u8` with all flags
    /// turned off (actually just [`PacketType::encode`]
    /// with [`PacketFlags::empty`] as a flags)
    pub const fn encode_ident(&self) -> u8 {
        self.encode(PacketFlags::empty())
    }

    /// Tries to decode packet type and packet flags from
    /// the integer value.
    pub fn try_decode(u: u8) -> Option<(PacketType, PacketFlags)> {
        let flags =
            unsafe { PacketFlags::from_bits_unchecked(u & 0b111) };
        PacketType::try_from(u >> 3)
            .ok()
            .map(|pt| (pt, flags))
    }

    /// Encodes packet type to the `u8` with the specified
    /// flags.
    pub const fn encode(&self, flags: PacketFlags) -> u8 {
        ((*self as u8) << 3) | flags.bits()
    }
}
