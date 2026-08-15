use std::collections::HashSet;

use crate::core::{BrokerError, FirewalldStatus, FwdBroker};
use crate::models::ZoneDetails;

pub(crate) async fn load_zones() -> Result<Vec<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_zones().await
}

pub(crate) async fn load_zone_details(zone_name: String) -> Result<ZoneDetails, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_zone_details(&zone_name).await
}

pub(crate) async fn load_default_zone() -> Result<String, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_default_zone().await
}

pub(crate) async fn load_active_zones() -> Result<HashSet<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_active_zones().await
}

pub(crate) async fn load_firewalld_status() -> Result<FirewalldStatus, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.firewalld_status().await
}
