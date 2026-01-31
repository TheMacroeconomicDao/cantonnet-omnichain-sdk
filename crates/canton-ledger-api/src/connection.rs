// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Connection management for ledger API

use crate::error::{LedgerError, LedgerResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use tracing::{debug, info, warn};

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Ledger endpoint URL
    pub endpoint: String,

    /// Maximum message size in bytes
    pub max_message_size: usize,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Keep-alive interval
    pub keep_alive_interval: Duration,

    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// HTTP/2 keep-alive while idle
    pub http2_keep_alive_interval: Duration,

    /// HTTP/2 keep-alive timeout
    pub http2_keep_alive_timeout: Duration,

    /// Whether to enable TCP keepalive
    pub tcp_keepalive: bool,

    /// TCP keepalive interval
    pub tcp_keepalive_interval: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:50051".to_string(),
            max_message_size: 64 * 1024 * 1024, // 64 MB
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(60),
            keep_alive_timeout: Duration::from_secs(5),
            tls: None,
            http2_keep_alive_interval: Duration::from_secs(30),
            http2_keep_alive_timeout: Duration::from_secs(10),
            tcp_keepalive: true,
            tcp_keepalive_interval: Duration::from_secs(60),
        }
    }
}

impl ConnectionConfig {
    /// Create a new connection configuration
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..Default::default()
        }
    }

    /// Set the maximum message size
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set the connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set TLS configuration
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Enable TLS with default configuration
    pub fn enable_tls(mut self) -> Self {
        self.tls = Some(TlsConfig::default());
        self
    }
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// CA certificate for server verification
    pub ca_cert: Option<Vec<u8>>,

    /// Client certificate for mutual TLS
    pub client_cert: Option<Vec<u8>>,

    /// Client private key for mutual TLS
    pub client_key: Option<Vec<u8>>,

    /// Domain name to verify against
    pub domain_name: Option<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            ca_cert: None,
            client_cert: None,
            client_key: None,
            domain_name: None,
        }
    }
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the CA certificate
    pub fn with_ca_cert(mut self, cert: Vec<u8>) -> Self {
        self.ca_cert = Some(cert);
        self
    }

    /// Set the client certificate
    pub fn with_client_cert(mut self, cert: Vec<u8>) -> Self {
        self.client_cert = Some(cert);
        self
    }

    /// Set the client private key
    pub fn with_client_key(mut self, key: Vec<u8>) -> Self {
        self.client_key = Some(key);
        self
    }

    /// Set the domain name
    pub fn with_domain_name(mut self, domain: impl Into<String>) -> Self {
        self.domain_name = Some(domain.into());
        self
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Reconnecting
    Reconnecting,
    /// Connection failed
    Failed,
}

/// Connection manager
pub struct ConnectionManager {
    config: ConnectionConfig,
    channel: Arc<RwLock<Option<Channel>>>,
    state: Arc<RwLock<ConnectionState>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            channel: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
        }
    }

    /// Get the connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == ConnectionState::Connected
    }

    /// Connect to the ledger
    pub async fn connect(&self) -> LedgerResult<Channel> {
        // Check if already connected
        {
            let channel = self.channel.read().await;
            if let Some(ref ch) = *channel {
                debug!("Using existing connection");
                return Ok(ch.clone());
            }
        }

        // Update state to connecting
        *self.state.write().await = ConnectionState::Connecting;

        info!("Connecting to ledger at {}", self.config.endpoint);

        // Build endpoint
        let mut endpoint = Endpoint::from_shared(self.config.endpoint.clone())
            .map_err(|e| LedgerError::InvalidConfiguration(format!("Invalid endpoint: {}", e)))?
            .connect_timeout(self.config.connect_timeout)
            .timeout(self.config.request_timeout)
            .keep_alive_timeout(self.config.keep_alive_timeout)
            .keep_alive_interval(self.config.keep_alive_interval)
            .http2_keep_alive_interval(self.config.http2_keep_alive_interval)
            .http2_keep_alive_timeout(self.config.http2_keep_alive_timeout)
            .tcp_keepalive(self.config.tcp_keepalive)
            .tcp_nodelay(true);

        // Configure TLS if specified
        if let Some(ref tls_config) = self.config.tls {
            let mut tls = ClientTlsConfig::new();

            if let Some(ref ca_cert) = tls_config.ca_cert {
                let cert = Certificate::from_pem(ca_cert);
                tls = tls.ca_certificate(cert);
            }

            if let Some(ref domain) = tls_config.domain_name {
                tls = tls.domain_name(domain);
            }

            endpoint = endpoint.tls_config(tls)
                .map_err(|e| LedgerError::InvalidConfiguration(format!("TLS configuration error: {}", e)))?;
        }

        // Connect
        let channel = endpoint.connect()
            .await
            .map_err(|e| LedgerError::ConnectionError(format!("Failed to connect: {}", e)))?;

        // Store channel
        *self.channel.write().await = Some(channel.clone());

        // Update state to connected
        *self.state.write().await = ConnectionState::Connected;

        info!("Successfully connected to ledger");

        Ok(channel)
    }

    /// Disconnect from the ledger
    pub async fn disconnect(&self) -> LedgerResult<()> {
        info!("Disconnecting from ledger");

        *self.channel.write().await = None;
        *self.state.write().await = ConnectionState::Disconnected;

        Ok(())
    }

    /// Reconnect to the ledger
    pub async fn reconnect(&self) -> LedgerResult<Channel> {
        warn!("Reconnecting to ledger");

        *self.state.write().await = ConnectionState::Reconnecting;

        self.disconnect().await?;
        self.connect().await
    }

    /// Get the current channel
    pub async fn channel(&self) -> LedgerResult<Channel> {
        let channel = self.channel.read().await;
        channel
            .as_ref()
            .cloned()
            .ok_or_else(|| LedgerError::ConnectionError("Not connected".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.endpoint, "http://localhost:50051");
        assert_eq!(config.max_message_size, 64 * 1024 * 1024);
    }

    #[test]
    fn test_connection_config_builder() {
        let config = ConnectionConfig::new("http://example.com:50051")
            .with_max_message_size(128 * 1024 * 1024)
            .with_connect_timeout(Duration::from_secs(20));

        assert_eq!(config.endpoint, "http://example.com:50051");
        assert_eq!(config.max_message_size, 128 * 1024 * 1024);
        assert_eq!(config.connect_timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(config.ca_cert.is_none());
        assert!(config.client_cert.is_none());
        assert!(config.client_key.is_none());
    }

    #[test]
    fn test_tls_config_builder() {
        let config = TlsConfig::new()
            .with_domain_name("example.com");

        assert_eq!(config.domain_name, Some("example.com".to_string()));
    }
}
