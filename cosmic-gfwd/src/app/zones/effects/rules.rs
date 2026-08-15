use crate::core::{BrokerError, FwdBroker};

pub(crate) async fn add_service(zone_name: String, service: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_service(&zone_name, &service).await
}

pub(crate) async fn add_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_port(&zone_name, &port, &protocol).await
}

pub(crate) async fn add_source_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_source_port(&zone_name, &port, &protocol).await
}

pub(crate) async fn add_forward_port(
    zone_name: String,
    port: String,
    protocol: String,
    to_port: String,
    to_addr: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker
        .add_forward_port(&zone_name, &port, &protocol, &to_port, &to_addr)
        .await
}

pub(crate) async fn add_interface(zone_name: String, interface: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_interface(&zone_name, &interface).await
}

pub(crate) async fn add_source(zone_name: String, source: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_source(&zone_name, &source).await
}

pub(crate) async fn add_icmp_block(zone_name: String, icmp: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_icmp_block(&zone_name, &icmp).await
}

pub(crate) async fn add_rich_rule(zone_name: String, rule: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_rich_rule(&zone_name, &rule).await
}

pub(crate) async fn remove_service(zone_name: String, service: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_service(&zone_name, &service).await
}

pub(crate) async fn remove_interface(
    zone_name: String,
    interface: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_interface(&zone_name, &interface).await
}

pub(crate) async fn remove_source(zone_name: String, source: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_source(&zone_name, &source).await
}

pub(crate) async fn remove_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_port(&zone_name, &port, &protocol).await
}

pub(crate) async fn remove_forward_port(
    zone_name: String,
    port: String,
    protocol: String,
    to_port: String,
    to_addr: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker
        .remove_forward_port(&zone_name, &port, &protocol, &to_port, &to_addr)
        .await
}

pub(crate) async fn remove_source_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker
        .remove_source_port(&zone_name, &port, &protocol)
        .await
}

pub(crate) async fn remove_icmp_block(zone_name: String, icmp: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_icmp_block(&zone_name, &icmp).await
}

pub(crate) async fn remove_rich_rule(zone_name: String, rule: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_rich_rule(&zone_name, &rule).await
}
