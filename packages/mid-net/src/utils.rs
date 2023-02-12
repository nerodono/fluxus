use std::io;

use crate::proto::PacketType;

pub trait FancyUtilExt {
    fn unitize_io(self) -> io::Result<()>;
}

impl<T> FancyUtilExt for io::Result<T> {
    fn unitize_io(self) -> io::Result<()> {
        self.map(|_| ())
    }
}

#[rustfmt::skip]
pub mod flags {
    pub const SHORT: u8        = 1 << 0;
    pub const SHORT_CLIENT: u8 = 1 << 1;
    pub const COMPRESSED: u8   = 1 << 2;

    /// Checks whether packet is short or not (size of length = 1 or 2)
    pub const fn is_short(f: u8) -> bool {
        (f & SHORT) != 0
    }

    /// Checks whether client id field is short or not
    /// (size of client id = 1 or 2)
    pub const fn is_short_client(f: u8) -> bool {
        (f & SHORT_CLIENT) != 0
    }

    /// Checks whether packet payload is compressed or not
    pub const fn is_compressed(f: u8) -> bool {
        (f & COMPRESSED) != 0
    }
}

pub const fn encode_fwd_header(
    client_id: u16,
    length: u16,
    compressed: bool,
) -> ([u8; 5], usize) {
    let mut flags: u8 = 0;
    let mut offset: usize = 1;
    let mut buf = [0; 5];

    offset += if client_id <= 0xff {
        flags |= flags::SHORT_CLIENT;
        buf[offset] = client_id as u8;

        1
    } else {
        buf[offset] = (client_id & 0xff) as u8;
        buf[offset + 1] = (client_id >> 8) as u8;

        2
    };
    offset += if length <= 0xff {
        flags |= flags::SHORT;
        buf[offset] = length as u8;

        1
    } else {
        buf[offset] = (length & 0xff) as u8;
        buf[offset + 1] = (length >> 8) as u8;

        2
    };

    if compressed {
        flags |= flags::COMPRESSED;
    }

    buf[0] = encode_type(PacketType::Forward as u8, flags);
    (buf, offset)
}

/// Encodes packet type to contain both type & flags.
pub const fn encode_type(pkt_type: u8, pkt_flags: u8) -> u8 {
    debug_assert!(pkt_flags <= 0b111);
    debug_assert!(pkt_type <= 0x1f);

    (pkt_type << 3) | pkt_flags
}

/// Same as [`encode_type`], but pkt_flags = 0
pub const fn ident_type(pkt_type: u8) -> u8 {
    encode_type(pkt_type, 0)
}
