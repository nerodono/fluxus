use crate::raw::{
    Packet,
    PacketFlags,
    PacketType,
};

pub fn encode_forward_header(
    client_id: u16,
    length: u16,
    mut flags: PacketFlags,
) -> ([u8; 5], u8) {
    let mut header = [0; 5];
    let mut offset = 1;

    offset += if client_id <= 0xff {
        header[offset] = client_id as u8;
        flags |= PacketFlags::SHORT_CLIENT;
        1
    } else {
        header[offset] = (client_id & 0xff) as u8;
        header[offset + 1] = (client_id >> 8) as u8;
        2
    };
    offset += if length <= 0xff {
        header[offset] = length as u8;
        flags |= PacketFlags::SHORT;
        1
    } else {
        header[offset] = (length & 0xff) as u8;
        header[offset + 1] = (length >> 8) as u8;
        2
    };

    header[0] = Packet::new(PacketType::Forward, flags).encode();

    (header, offset as u8)
}
