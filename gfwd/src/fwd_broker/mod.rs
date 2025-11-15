use std::{collections::HashMap, sync::Arc};

use relm4::tokio::sync::OnceCell;

use crate::error::GfwdError;

mod backend;
#[cfg(feature = "dbus-backend")]
mod dbus;
#[cfg(feature = "mock-backend")]
mod mock;

use backend::build_backend;
pub use backend::{FirewalldBackend, ZoneSettings, ZoneTarget};

pub struct FwdBroker {
    backend: Arc<dyn FirewalldBackend>,
}

impl PartialEq for FwdBroker {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
    pub async fn get_broker() -> &'static FwdBroker {
        BROKER
            .get_or_init(|| async {
                let backend = build_backend().await;
                FwdBroker { backend }
            })
            .await
    }

    pub async fn get_zones(&self) -> Result<Vec<String>, GfwdError> {
        self.backend.get_zones().await
    }

    pub async fn get_active_zones(
        &self,
    ) -> Result<HashMap<String, HashMap<String, Vec<String>>>, GfwdError> {
        self.backend.get_active_zones().await
    }

    pub async fn get_default_zone(&self) -> Result<String, GfwdError> {
        self.backend.get_default_zone().await
    }

    pub async fn get_zone_settings(&self, zone_name: &str) -> Result<ZoneSettings, GfwdError> {
        self.backend.get_zone_settings(zone_name).await
    }

    pub async fn add_zone(&self, settings: ZoneSettings) -> Result<(), GfwdError> {
        self.backend.add_zone(settings).await
    }

    pub async fn remove_zone(&self, zone_name: &str) -> Result<(), GfwdError> {
        self.backend.remove_zone(zone_name).await
    }

    pub async fn add_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError> {
        self.backend.add_port(zone_name, port, protocol).await
    }

    pub async fn remove_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError> {
        self.backend.remove_port(zone_name, port, protocol).await
    }

    pub async fn add_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError> {
        self.backend
            .add_forward_port(zone_name, port, protocol, to_port, to_addr)
            .await
    }

    pub async fn remove_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError> {
        self.backend
            .remove_forward_port(zone_name, port, protocol, to_port, to_addr)
            .await
    }

    pub async fn is_firewalld_active(&self) -> Result<bool, GfwdError> {
        self.backend.is_firewalld_active().await
    }

    pub async fn start_firewalld(&self) -> Result<(), GfwdError> {
        self.backend.start_firewalld().await
    }

    pub async fn stop_firewalld(&self) -> Result<(), GfwdError> {
        self.backend.stop_firewalld().await
    }
}
