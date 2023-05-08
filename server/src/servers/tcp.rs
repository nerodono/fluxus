use super::channels_map::ChannelsMap;
use crate::data::commands::tcp::TcpSlaveCommand;

#[derive(Default)]
pub struct TcpServer {
    pub clients: ChannelsMap<TcpSlaveCommand>,
}
