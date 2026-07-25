//! Firewalld root, runtime-zone, and selected permanent-zone signal streams.

use futures_util::{StreamExt, stream::BoxStream};
use gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy;
use gfwd_bus::config_zone::ConfigZoneProxy;
use gfwd_bus::firewalld1::FirewallD1Proxy;
use gfwd_bus::zone::ZoneProxy;

use crate::core::ConfigurationEvent;

use super::{BrokerError, FwdBroker};

impl FwdBroker {
    /// Produce a broker-owned stream of global, runtime-zone, and selected
    /// permanent-zone configuration events.
    pub fn configuration_events(
        &self,
        selected_zone: Option<String>,
    ) -> BoxStream<'static, Result<ConfigurationEvent, BrokerError>> {
        let conn = self.conn.clone();
        Box::pin(async_stream::stream! {
            let root = match FirewallD1Proxy::new(&conn).await {
                Ok(proxy) => proxy,
                Err(error) => {
                    yield Err(BrokerError::from(error));
                    return;
                }
            };
            let runtime = match ZoneProxy::new(&conn).await {
                Ok(proxy) => proxy,
                Err(error) => {
                    yield Err(BrokerError::from(error));
                    return;
                }
            };
            let mut root_signals = match root.inner().receive_all_signals().await {
                Ok(stream) => stream,
                Err(error) => {
                    yield Err(BrokerError::from(error));
                    return;
                }
            };
            let mut runtime_signals = match runtime.inner().receive_all_signals().await {
                Ok(stream) => stream,
                Err(error) => {
                    yield Err(BrokerError::from(error));
                    return;
                }
            };

            let mut permanent_signals = if let Some(zone_name) = selected_zone.as_deref() {
                let config = match ConfigFirewalld1Proxy::new(&conn).await {
                    Ok(proxy) => proxy,
                    Err(error) => {
                        yield Err(BrokerError::from(error));
                        return;
                    }
                };
                let path = match config.get_zone_by_name(zone_name).await {
                    Ok(path) => path,
                    Err(error) => {
                        yield Err(BrokerError::from(error));
                        return;
                    }
                };
                let proxy = match ConfigZoneProxy::builder(&conn).path(path) {
                    Ok(builder) => match builder.build().await {
                        Ok(proxy) => proxy,
                        Err(error) => {
                            yield Err(BrokerError::from(error));
                            return;
                        }
                    },
                    Err(error) => {
                        yield Err(BrokerError::from(error));
                        return;
                    }
                };
                match proxy.inner().receive_all_signals().await {
                    Ok(stream) => Some(stream),
                    Err(error) => {
                        yield Err(BrokerError::from(error));
                        return;
                    }
                }
            } else {
                None
            };

            loop {
                tokio::select! {
                    signal = root_signals.next() => {
                        let Some(message) = signal else {
                            yield Err(BrokerError::new("firewalld root signal stream ended"));
                            return;
                        };
                        if signal_member(&message).as_deref() == Some("Reloaded") {
                            yield Ok(ConfigurationEvent::Reloaded);
                        }
                    }
                    signal = runtime_signals.next() => {
                        let Some(message) = signal else {
                            yield Err(BrokerError::new("firewalld runtime-zone signal stream ended"));
                            return;
                        };
                        let Some(selected) = selected_zone.as_deref() else {
                            continue;
                        };
                        if matches!(
                            first_signal_string(&message),
                            Ok(zone) if zone == selected
                        ) {
                            yield Ok(ConfigurationEvent::RuntimeZoneChanged {
                                zone: selected.to_string(),
                            });
                        }
                    }
                    signal = async {
                        match permanent_signals.as_mut() {
                            Some(stream) => stream.next().await,
                            None => futures_util::future::pending().await,
                        }
                    } => {
                        let Some(message) = signal else {
                            yield Err(BrokerError::new("selected permanent-zone signal stream ended"));
                            return;
                        };
                        let Some(zone) = selected_zone.as_deref() else {
                            continue;
                        };
                        match signal_member(&message).as_deref() {
                            Some("Updated") => {
                                yield Ok(ConfigurationEvent::PermanentZoneUpdated {
                                    zone: zone.to_string(),
                                });
                            }
                            Some("Removed") => {
                                yield Ok(ConfigurationEvent::PermanentZoneRemoved {
                                    zone: zone.to_string(),
                                });
                            }
                            Some("Renamed") => match first_signal_string(&message) {
                                Ok(new_zone) => {
                                    yield Ok(ConfigurationEvent::PermanentZoneRenamed {
                                        old_zone: zone.to_string(),
                                        new_zone,
                                    });
                                }
                                Err(error) => {
                                    yield Err(error);
                                    return;
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        })
    }
}

fn signal_member(message: &zbus::Message) -> Option<String> {
    message
        .header()
        .member()
        .map(|member| member.as_str().to_string())
}

fn first_signal_string(message: &zbus::Message) -> Result<String, BrokerError> {
    let body = message.body();
    let structure: zvariant::Structure<'_> = body
        .deserialize()
        .map_err(|error| BrokerError::new(error.to_string()))?;
    let value = structure
        .fields()
        .first()
        .ok_or_else(|| BrokerError::new("firewalld signal omitted its zone argument"))?;
    let value: &str = value
        .downcast_ref()
        .map_err(|error| BrokerError::new(error.to_string()))?;
    Ok(value.to_string())
}
