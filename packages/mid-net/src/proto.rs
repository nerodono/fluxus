use integral_enum::IntegralEnum;

#[derive(IntegralEnum)]
pub enum PacketType {
    Failure = 0,
    Ping = 1,
}
