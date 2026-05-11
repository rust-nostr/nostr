//! Relay proxy configuration.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use nostr::RelayUrl;

#[derive(Clone)]
enum InnerProxy {
    All(SocketAddr),
    #[allow(clippy::type_complexity)]
    Custom(Arc<dyn Fn(&RelayUrl) -> Option<SocketAddr> + Send + Sync + 'static>),
}

/// SOCKS5 proxy policy for relay connections.
///
/// A proxy policy decides whether a relay connection should use a SOCKS5 proxy
/// and which proxy address should be used. The policy is evaluated separately
/// for each relay URL when the relay connects.
///
/// Use [`Proxy::all`] to route every relay through the same proxy,
/// [`Proxy::onion`] to route only `.onion` relays, or [`Proxy::custom`] for
/// application-specific routing.
#[derive(Clone)]
pub struct Proxy(InnerProxy);

impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            InnerProxy::All(addr) => f
                .debug_tuple("Proxy")
                .field(&format_args!("All({})", addr))
                .finish(),
            InnerProxy::Custom(_) => f.debug_tuple("Proxy").field(&"Custom").finish(),
        }
    }
}

impl Proxy {
    /// Use a SOCKS5 proxy for all relay connections.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    /// # use nostr_sdk::prelude::*;
    /// let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    /// let proxy = Proxy::all(addr);
    /// let client = Client::builder().proxy(proxy).build();
    /// ```
    #[inline]
    pub fn all(addr: SocketAddr) -> Self {
        Self(InnerProxy::All(addr))
    }

    /// Use a SOCKS5 proxy only for `.onion` relay connections.
    ///
    /// This is a convenience
    /// wrapper around [`Proxy::custom`] for the common Tor SOCKS5 proxy setup.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    /// # use nostr_sdk::prelude::*;
    /// let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    /// let proxy = Proxy::onion(addr);
    /// let client = Client::builder().proxy(proxy).build();
    /// ```
    #[inline]
    pub fn onion(addr: SocketAddr) -> Self {
        Self::custom(move |relay_url| {
            if relay_url.is_onion() {
                Some(addr)
            } else {
                None
            }
        })
    }

    /// Use a custom SOCKS5 proxy policy.
    ///
    /// The callback receives the relay URL and returns the proxy address to use.
    /// Returning [`None`] means that the relay should use a direct connection.
    ///
    /// Use this when proxy routing depends on application-specific rules, such
    /// as selected domains, user settings, or multiple proxy endpoints.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    /// # use nostr_sdk::prelude::*;
    /// let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    /// let client = Client::builder()
    ///     .proxy(Proxy::custom(move |relay_url| {
    ///         if relay_url.domain() == Some("example.com") {
    ///             Some(addr)
    ///         } else {
    ///             None
    ///         }
    ///     }))
    ///     .build();
    /// ```
    #[inline]
    pub fn custom<F>(fun: F) -> Self
    where
        F: Fn(&RelayUrl) -> Option<SocketAddr> + Send + Sync + 'static,
    {
        Self(InnerProxy::Custom(Arc::new(fun)))
    }

    #[inline]
    pub(crate) fn get_addr(&self, url: &RelayUrl) -> Option<SocketAddr> {
        match &self.0 {
            InnerProxy::All(addr) => Some(*addr),
            InnerProxy::Custom(fun) => fun(url),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use super::*;

    const ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));

    #[test]
    fn test_proxy_all() {
        let proxy: Proxy = Proxy::all(ADDR);

        let clearnet_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let onion_url =
            RelayUrl::parse("ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion")
                .unwrap();

        assert_eq!(proxy.get_addr(&clearnet_url), Some(ADDR));
        assert_eq!(proxy.get_addr(&onion_url), Some(ADDR));
    }

    #[test]
    fn test_proxy_onion() {
        let proxy: Proxy = Proxy::onion(ADDR);

        let clearnet_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let onion_url =
            RelayUrl::parse("ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion")
                .unwrap();

        assert!(proxy.get_addr(&clearnet_url).is_none());
        assert_eq!(proxy.get_addr(&onion_url), Some(ADDR));
    }

    #[test]
    fn test_proxy_custom() {
        let proxy: Proxy = Proxy::custom(move |url| {
            if url.domain() == Some("example.com") {
                Some(ADDR)
            } else {
                None
            }
        });

        let damus_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let example_url = RelayUrl::parse("wss://example.com").unwrap();

        assert!(proxy.get_addr(&damus_url).is_none());
        assert_eq!(proxy.get_addr(&example_url), Some(ADDR));
    }
}
