use rstest::rstest;

use crate::{
    raw::{
        Packet,
        PacketFlags,
        PacketType,
    },
    utils::encode_forward_header,
};

// Pretty ugly test, but needed for correctness
#[rstest]
#[case(PacketType::Ping, PacketFlags::empty(), ((PacketType::Ping as u8) << 3))]
#[case(PacketType::Ping, PacketFlags::COMPRESSED,
    ((PacketType::Ping as u8) << 3) | PacketFlags::COMPRESSED.bits())
]
#[case(PacketType::Ping, PacketFlags::all(),
    ((PacketType::Ping as u8) << 3) | PacketFlags::all().bits())]
#[case(PacketType::Ping, PacketFlags::SHORT | PacketFlags::SHORT_CLIENT,
    ((PacketType::Ping as u8) << 3) | (PacketFlags::SHORT | PacketFlags::SHORT_CLIENT).bits())]
fn test_encode_packet_type(
    #[case] ty: PacketType,
    #[case] flags: PacketFlags,
    #[case] should_be: u8,
) {
    assert_eq!(Packet::new(ty, flags).encode(), should_be);
}

#[rstest]
#[case(false, 1, 1, &[
    Packet::new(PacketType::Forward, PacketFlags::SHORT | PacketFlags::SHORT_CLIENT).encode(),
    1,
    1,
])]
#[case(true, 1, 1, &[
    Packet::new(PacketType::Forward, PacketFlags::all()).encode(),
    1,
    1
])]
#[case(true, 1, 256, &[
    Packet::new(PacketType::Forward, PacketFlags::SHORT_CLIENT | PacketFlags::COMPRESSED).encode(),
    1,
    0,
    1
])]
#[case(false, 256, 256, &[
    Packet::new(PacketType::Forward, PacketFlags::empty()).encode(),
    0,
    1,
    0,
    1,
])]
#[case(false, 256, 1, &[
    Packet::new(PacketType::Forward, PacketFlags::SHORT).encode(),
    0,
    1,
    1
])]
#[case(true, 256, 1, &[
    Packet::new(PacketType::Forward, PacketFlags::SHORT | PacketFlags::COMPRESSED).encode(),
    0,
    1,
    1
])]
fn test_forward_encode(
    #[case] compressed: bool,
    #[case] client_id: u16,
    #[case] length: u16,
    #[case] should_be: &[u8],
) {
    let real_output = encode_forward_header(
        client_id,
        length,
        if compressed {
            PacketFlags::COMPRESSED
        } else {
            PacketFlags::empty()
        },
    );
    let real_slice = &real_output.0[..(real_output.1 as usize)];
    assert_eq!(real_slice, should_be);
}
