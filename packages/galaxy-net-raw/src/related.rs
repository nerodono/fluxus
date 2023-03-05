use integral_enum::IntegralEnum;

#[derive(IntegralEnum)]
#[repr(u8)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
    Http = 2,
}
