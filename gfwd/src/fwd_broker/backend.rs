use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::error::GfwdError;

#[cfg(feature = "dbus-backend")]
use super::dbus::DbusFirewalld;
#[cfg(feature = "mock-backend")]
use super::mock::MockFirewalld;

#[cfg(all(feature = "mock-backend", not(feature = "dbus-backend")))]
pub async fn build_backend() -> Arc<dyn FirewalldBackend> {
    Arc::new(MockFirewalld::new())
}

#[cfg(all(feature = "dbus-backend", not(feature = "mock-backend")))]
pub async fn build_backend() -> Arc<dyn FirewalldBackend> {
    match DbusFirewalld::connect().await {
        Ok(client) => Arc::new(client),
        Err(err) => panic!("Failed to connect to firewalld via D-Bus: {err}"),
    }
}

#[cfg(all(feature = "mock-backend", feature = "dbus-backend"))]
pub async fn build_backend() -> Arc<dyn FirewalldBackend> {
    use std::env;

    if matches!(
        env::var("GFWD_BACKEND"),
        Ok(value) if value.eq_ignore_ascii_case("mock")
    ) {
        return Arc::new(MockFirewalld::new());
    }

    match DbusFirewalld::connect().await {
        Ok(client) => Arc::new(client),
        Err(err) => {
            eprintln!(
                "Failed to connect to firewalld via D-Bus ({err}). Falling back to mock backend."
            );
            Arc::new(MockFirewalld::new())
        }
    }
}

#[cfg(all(not(feature = "dbus-backend"), not(feature = "mock-backend")))]
compile_error!("Either 'dbus-backend' or 'mock-backend' feature must be enabled");

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ZoneSettings {
    pub version: String,
    pub name: String,
    pub description: String,
    pub unused: bool,
    pub target: ZoneTarget,
    pub services: Vec<String>,
    pub ports: Vec<(String, String)>,
    pub icmp_blocks: Vec<String>,
    pub masquerade: bool,
    pub forward_ports: Vec<(String, String, String, String)>,
    pub interfaces: Vec<String>,
    pub sources: Vec<String>,
    pub rich_rules: Vec<String>,
    pub protocols: Vec<String>,
    pub source_ports: Vec<(String, String)>,
}

#[derive(Debug, Default, derive_more::Display, Clone, PartialEq)]
#[allow(unused)]
pub enum ZoneTarget {
    #[default]
    #[display("default")]
    Default,
    #[display("ACCEPT")]
    Accept,
    #[display("DROP")]
    Drop,
    #[display("REJECT")]
    Reject,
}

#[async_trait]
pub trait FirewalldBackend: Send + Sync {
    async fn get_zones(&self) -> Result<Vec<String>, GfwdError>;

    async fn get_active_zones(
        &self,
    ) -> Result<HashMap<String, HashMap<String, Vec<String>>>, GfwdError>;

    async fn get_default_zone(&self) -> Result<String, GfwdError>;

    async fn get_zone_settings(&self, zone_name: &str) -> Result<ZoneSettings, GfwdError>;

    async fn add_zone(&self, settings: ZoneSettings) -> Result<(), GfwdError>;

    async fn remove_zone(&self, zone_name: &str) -> Result<(), GfwdError>;

    async fn add_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError>;

    async fn remove_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError>;

    async fn add_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError>;

    async fn remove_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError>;

    async fn is_firewalld_active(&self) -> Result<bool, GfwdError>;

    async fn start_firewalld(&self) -> Result<(), GfwdError>;

    async fn stop_firewalld(&self) -> Result<(), GfwdError>;
}
