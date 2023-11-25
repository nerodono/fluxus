use std::net::SocketAddr;

entity! {
    struct ServerConfig {
        name: String,
        protocols: ProtocolsConfig
    }
}

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
