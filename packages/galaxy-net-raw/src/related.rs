use integral_enum::IntegralEnum;

#[derive(IntegralEnum)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[repr(u8)]
pub enum CompressionMethod {
    #[cfg_attr(feature = "serde", serde(rename = "zstd"))]
    ZStd = 0,
}

#[derive(IntegralEnum)]
#[repr(u8)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
    Http = 2,
}
