//! Optional lifecycle observations. Observers never authorize a mapping.
use std::{
    fmt,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

/// The gateway protocol chosen for an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingProtocol {
    Pcp,
    NatPmp,
    Upnp,
}

/// An actual service transition, including renewals that keep the same address.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Event {
    Discovering,
    Gateway {
        local_ip: Ipv4Addr,
        gateway: Ipv4Addr,
    },
    Discovered(super::ProbeOutput),
    DiscoveryFailed(String),
    Requesting {
        protocol: MappingProtocol,
        gateway: Ipv4Addr,
        renewal: bool,
    },
    Active {
        protocol: MappingProtocol,
        external: SocketAddrV4,
        lifetime: Duration,
    },
    MappingFailed(String),
    Expired(SocketAddrV4),
    Cancelled,
    Releasing(SocketAddrV4),
    /// `confirmed` is false for PCP/NAT-PMP, whose deletion is sent without
    /// awaiting an acknowledgement, and true for a successful UPnP response.
    Released {
        external: SocketAddrV4,
        confirmed: bool,
    },
    ReleaseFailed {
        external: SocketAddrV4,
        error: String,
    },
}

/// A synchronous, non-blocking observer. Callbacks must not panic or call back
/// into the service. Without an observer there is no event queue or task.
#[derive(Clone, Default)]
pub struct Observer(Option<Arc<dyn Fn(Event) + Send + Sync>>);

impl Observer {
    pub fn new(callback: impl Fn(Event) + Send + Sync + 'static) -> Self {
        Self(Some(Arc::new(callback)))
    }
    pub(crate) fn emit(&self, event: Event) {
        if let Some(callback) = &self.0 {
            callback(event);
        }
    }
}
impl fmt::Debug for Observer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Observer")
            .field("enabled", &self.0.is_some())
            .finish()
    }
}

pub(crate) fn detail(error: impl fmt::Display) -> String {
    error.to_string().chars().take(256).collect()
}
