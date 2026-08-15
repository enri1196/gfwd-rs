use crate::core::{BrokerError, FwdBroker};
use crate::models::ZoneTarget;

pub(crate) async fn set_masquerade(zone_name: String, enabled: bool) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.set_masquerade(&zone_name, enabled).await
}

pub(crate) async fn set_icmp_block_inversion(
    zone_name: String,
    enabled: bool,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.set_icmp_block_inversion(&zone_name, enabled).await
}

pub(crate) async fn set_default_zone(zone_name: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.set_default_zone(&zone_name).await
}

pub(crate) async fn add_zone(
    name: String,
    description: String,
    target: ZoneTarget,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_zone(&name, &description, &target).await
}

pub(crate) async fn remove_zone(zone_name: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_zone(&zone_name).await
}
