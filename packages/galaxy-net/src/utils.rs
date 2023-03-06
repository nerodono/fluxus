use galaxy_net_raw::packet_type::{
    PacketFlags,
    PacketType,
};

/// Encode forward packet header
pub fn encode_forward_header(
    length: u16,
    client_id: u16,
    mut flags: PacketFlags,
) -> ([u8; 5], u8) {
    let mut buf = [0; 5];
    let mut offset: usize = 1;

    offset += if client_id <= 0xff {
        flags |= PacketFlags::SHORT_C;
        buf[offset] = client_id as u8;
        1
    } else {
        buf[offset] = (client_id & 0xff) as u8;
        buf[offset + 1] = (client_id >> 8) as u8;
        2
    };

    offset += if length <= 0xff {
        flags |= PacketFlags::SHORT;
        buf[offset] = length as u8;
        1
    } else {
        buf[offset] = (length & 0xff) as u8;
        buf[offset + 1] = (length >> 8) as u8;
        2
    };

    buf[0] = PacketType::Forward.encode(flags);
    (buf, offset as u8)
}
