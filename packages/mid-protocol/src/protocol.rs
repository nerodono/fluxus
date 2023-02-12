use integral_enum::IntegralEnum;

#[derive(IntegralEnum)]
pub enum PacketType {
    Err,

    Ping,
}
