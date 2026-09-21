//! Firewalld root, runtime-zone, and selected permanent-zone signal streams.

use std::time::Duration;

use futures_util::{StreamExt, stream::BoxStream};
use gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy;
use gfwd_bus::config_zone::ConfigZoneProxy;
use gfwd_bus::firewalld1::FirewallD1Proxy;
use gfwd_bus::zone::ZoneProxy;
use zbus::fdo::DBusProxy;

use crate::core::ConfigurationEvent;

use super::{BrokerError, FwdBroker};

const FIREWALLD_DESTINATION: &str = "org.fedoraproject.FirewallD1";
const OWNER_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const OWNER_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) enum ConfigurationSignal {
    Changed(ConfigurationEvent),
    Healthy,
}

pub(crate) struct ConfigurationWatchSession {
    pub(crate) signals: BoxStream<'static, Result<ConfigurationSignal, BrokerError>>,
}

impl FwdBroker {
    /// Establish a complete broker-owned configuration watch session.
    pub(crate) async fn open_configuration_watch(
        &self,
        selected_zone: Option<String>,
    ) -> Result<ConfigurationWatchSession, BrokerError> {
        let conn = self.conn.clone();
        let dbus = DBusProxy::new(&conn).await.map_err(BrokerError::from)?;
        let firewalld_name = zbus::names::BusName::try_from(FIREWALLD_DESTINATION)
            .map_err(|error| BrokerError::new(error.to_string()))?;
        let initial_owner = dbus
            .get_name_owner(firewalld_name.clone())
            .await
            .map_err(|error| BrokerError::new(error.to_string()))?
            .to_string();

        let root = FirewallD1Proxy::builder(&conn)
            .destination(initial_owner.as_str())
            .map_err(BrokerError::from)?
            .build()
            .await
            .map_err(BrokerError::from)?;
        let runtime = ZoneProxy::builder(&conn)
            .destination(initial_owner.as_str())
            .map_err(BrokerError::from)?
            .build()
            .await
            .map_err(BrokerError::from)?;
        let mut root_signals = root
            .inner()
            .receive_all_signals()
            .await
            .map_err(BrokerError::from)?;
        let mut runtime_signals = runtime
            .inner()
            .receive_all_signals()
            .await
            .map_err(BrokerError::from)?;

        let mut permanent_signals = if let Some(zone_name) = selected_zone.as_deref() {
            let config = ConfigFirewalld1Proxy::builder(&conn)
                .destination(initial_owner.as_str())
                .map_err(BrokerError::from)?
                .build()
                .await
                .map_err(BrokerError::from)?;
            let zone_names = config.get_zone_names().await.map_err(BrokerError::from)?;
            if selected_zone_exists(Some(zone_name), &zone_names) {
                let path = config
                    .get_zone_by_name(zone_name)
                    .await
                    .map_err(BrokerError::from)?;
                let proxy = ConfigZoneProxy::builder(&conn)
                    .destination(initial_owner.clone())
                    .map_err(BrokerError::from)?
                    .path(path)
                    .map_err(BrokerError::from)?
                    .build()
                    .await
                    .map_err(BrokerError::from)?;
                Some(
                    proxy
                        .inner()
                        .receive_all_signals()
                        .await
                        .map_err(BrokerError::from)?,
                )
            } else {
                None
            }
        } else {
            None
        };

        let current_owner = dbus
            .get_name_owner(firewalld_name.clone())
            .await
            .map_err(|error| BrokerError::new(error.to_string()))?
            .to_string();
        ensure_same_owner(&initial_owner, &current_owner)?;

        let mut owner_checks = tokio::time::interval_at(
            tokio::time::Instant::now() + OWNER_CHECK_INTERVAL,
            OWNER_CHECK_INTERVAL,
        );
        owner_checks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let initial_owner_for_stream = initial_owner.clone();
        let signals = Box::pin(async_stream::stream! {
            loop {
                tokio::select! {
                    signal = root_signals.next() => {
                        let Some(message) = signal else {
                            yield Err(BrokerError::new("firewalld root signal stream ended"));
                            return;
                        };
                        if signal_member(&message).as_deref() == Some("Reloaded") {
                            yield Ok(ConfigurationSignal::Changed(ConfigurationEvent::Reloaded));
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
                            yield Ok(ConfigurationSignal::Changed(ConfigurationEvent::RuntimeZoneChanged {
                                zone: selected.to_string(),
                            }));
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
                                yield Ok(ConfigurationSignal::Changed(ConfigurationEvent::PermanentZoneUpdated {
                                    zone: zone.to_string(),
                                }));
                            }
                            Some("Removed") => {
                                yield Ok(ConfigurationSignal::Changed(ConfigurationEvent::PermanentZoneRemoved {
                                    zone: zone.to_string(),
                                }));
                            }
                            Some("Renamed") => match first_signal_string(&message) {
                                Ok(new_zone) => {
                                    yield Ok(ConfigurationSignal::Changed(ConfigurationEvent::PermanentZoneRenamed {
                                        old_zone: zone.to_string(),
                                        new_zone,
                                    }));
                                }
                                Err(error) => {
                                    yield Err(error);
                                    return;
                                }
                            },
                            _ => {}
                        }
                    }
                    _ = owner_checks.tick() => {
                        let current_owner = match tokio::time::timeout(
                            OWNER_CHECK_TIMEOUT,
                            dbus.get_name_owner(firewalld_name.clone()),
                        ).await {
                            Ok(Ok(owner)) => owner.to_string(),
                            Ok(Err(error)) => {
                                yield Err(BrokerError::new(error.to_string()));
                                return;
                            }
                            Err(_) => {
                                yield Err(BrokerError::new("firewalld owner check timed out"));
                                return;
                            }
                        };
                        if let Err(error) = ensure_same_owner(&initial_owner_for_stream, &current_owner) {
                            yield Err(error);
                            return;
                        }
                        yield Ok(ConfigurationSignal::Healthy);
                    }
                }
            }
        });

        self.publish_configuration_connection().await;
        Ok(ConfigurationWatchSession { signals })
    }

    /// Produce a broker-owned stream of global, runtime-zone, and selected
    /// permanent-zone configuration events.
    pub fn configuration_events(
        &self,
        selected_zone: Option<String>,
    ) -> BoxStream<'static, Result<ConfigurationEvent, BrokerError>> {
        let broker = self.clone();
        Box::pin(async_stream::stream! {
            let mut session = match broker.open_configuration_watch(selected_zone).await {
                Ok(session) => session,
                Err(error) => {
                    yield Err(error);
                    return;
                }
            };
            while let Some(signal) = session.signals.next().await {
                match signal {
                    Ok(ConfigurationSignal::Changed(event)) => yield Ok(event),
                    Ok(ConfigurationSignal::Healthy) => {}
                    Err(error) => {
                        yield Err(error);
                        return;
                    }
                }
            }
        })
    }
}

fn ensure_same_owner(initial_owner: &str, current_owner: &str) -> Result<(), BrokerError> {
    if initial_owner == current_owner {
        Ok(())
    } else {
        Err(BrokerError::new("firewalld owner changed"))
    }
}

fn selected_zone_exists(selected_zone: Option<&str>, zone_names: &[String]) -> bool {
    selected_zone.is_none_or(|selected| zone_names.iter().any(|name| name == selected))
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

#[cfg(test)]
mod tests {
    use super::{ensure_same_owner, selected_zone_exists};

    #[test]
    fn same_owner_keeps_session_alive() {
        assert!(ensure_same_owner(":1.42", ":1.42").is_ok());
    }

    #[test]
    fn changed_owner_invalidates_session() {
        let error = ensure_same_owner(":1.42", ":1.43").expect_err("owner change must fail");

        assert_eq!(error.to_string(), "firewalld owner changed");
    }

    #[test]
    fn selected_zone_existence_matches_optional_selection() {
        let zones = vec!["public".to_string(), "work".to_string()];

        assert!(selected_zone_exists(None, &zones));
        assert!(selected_zone_exists(Some("public"), &zones));
        assert!(!selected_zone_exists(Some("deleted"), &zones));
    }
}
