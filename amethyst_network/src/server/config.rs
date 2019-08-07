use laminar::Config;
use std::net::SocketAddr;

#[derive(Clone, new)]
/// The configuration used for the networking system.
#[allow(missing_debug_implementations)] // TODO: revisit this. Laminar is missing Debug on a lot of things
pub struct ServerConfig {
    /// Address at which the UDP server will listen for incoming packets.
    pub udp_socket_addr: SocketAddr,
    /// Specifies what the maximal packets that could be handled by the server.
    /// This value is meant for preventing some loops to read infinitely long when many packets are send and received.
    /// This value is by default 5000.
    pub max_throughput: u16,
    // If enabled a `NetConnection` will be automatically added to the world when a client connects.
    /// Make this property 'false' you prevent this behaviour.
    /// This property is enabled by default.
    pub create_net_connection_on_connect: bool,
    /// Allows you to configure laminar its behaviour.
    pub laminar_config: Config,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            // by passing in :0 port the OS will give an available port.
            udp_socket_addr: "0.0.0.0:0".parse().unwrap(),
            max_throughput: 5000,
            create_net_connection_on_connect: true,
            laminar_config: Config::default(),
        }
    }
}
