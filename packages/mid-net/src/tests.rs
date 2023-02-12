use rstest::rstest;

use crate::utils::{
    encode_type,
    ident_type,
};

#[rstest]
#[case(0, 0)]
#[case(1, 8)]
#[case(2, 16)]
#[case(3, 24)]
fn ident_encode_test(#[case] pkt_type: u8, #[case] expected: u8) {
    assert_eq!(expected, ident_type(pkt_type));
}

#[rstest]
#[case(0, 0, 0)]
#[case(1, 0, 8)]
#[case(1, 1, 9)]
#[case(1, 2, 10)]
#[case(2, 0, 16)]
#[case(2, 1, 17)]
#[case(2, 2, 18)]
fn encoding_test(
    #[case] pkt_type: u8,
    #[case] pkt_flags: u8,
    #[case] expected: u8,
) {
    assert_eq!(expected, encode_type(pkt_type, pkt_flags));
}
