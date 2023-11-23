use std::net::SocketAddr;

entity! {
    struct ProtocolsConfig {
        #[cfg(feature = "tcpflux")]
        tcp_flux: TcpFlux
    }

    #[cfg(feature = "tcpflux")]
    struct TcpFlux {
        listen: SocketAddr,
    }
}
