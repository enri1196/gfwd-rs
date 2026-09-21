use std::time::Duration;

use async_stream::stream;
use futures_util::{StreamExt, future::BoxFuture, stream::BoxStream};
use tokio::time::{Instant, sleep, timeout};

use super::{
    broker::{BrokerError, ConfigurationSignal, ConfigurationWatchSession, FwdBroker},
    events::ConfigurationEvent,
};

#[cfg(test)]
mod tests;

const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const STABLE_CONNECTION_DURATION: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub(crate) enum ConfigurationWatchEvent {
    Ready,
    Changed(ConfigurationEvent),
    Unavailable(String),
}

pub(crate) trait ConfigurationEventSource: Send + Sync + 'static {
    fn connect(
        &self,
        selected_zone: Option<String>,
    ) -> BoxFuture<'_, Result<ConfigurationWatchSession, BrokerError>>;
}

pub(crate) struct BrokerConfigurationEventSource;

pub(crate) struct ConfigurationEventWatcher<S = BrokerConfigurationEventSource> {
    source: S,
    selected_zone: Option<String>,
}

struct RetryBackoff {
    next_delay: Duration,
}

impl ConfigurationEventSource for BrokerConfigurationEventSource {
    fn connect(
        &self,
        selected_zone: Option<String>,
    ) -> BoxFuture<'_, Result<ConfigurationWatchSession, BrokerError>> {
        Box::pin(async move {
            let broker = FwdBroker::connect().await?;
            broker.open_configuration_watch(selected_zone).await
        })
    }
}

impl ConfigurationEventWatcher<BrokerConfigurationEventSource> {
    pub(crate) fn new(selected_zone: Option<String>) -> Self {
        Self {
            source: BrokerConfigurationEventSource,
            selected_zone,
        }
    }
}

#[cfg(test)]
impl<S> ConfigurationEventWatcher<S> {
    fn with_source(selected_zone: Option<String>, source: S) -> Self {
        Self {
            source,
            selected_zone,
        }
    }
}

impl<S> ConfigurationEventWatcher<S>
where
    S: ConfigurationEventSource,
{
    pub(crate) fn into_stream(self) -> BoxStream<'static, ConfigurationWatchEvent> {
        let source = self.source;
        let selected_zone = self.selected_zone;

        Box::pin(stream! {
            let mut backoff = RetryBackoff::new();

            loop {
                let failure = match timeout(
                    CONNECT_TIMEOUT,
                    source.connect(selected_zone.clone()),
                )
                .await
                {
                    Ok(Ok(session)) => {
                        {
                            let mut session = session;
                            let ready_since = Instant::now();
                            let mut stable = false;

                            yield ConfigurationWatchEvent::Ready;

                            loop {
                                match session.signals.next().await {
                                    Some(Ok(ConfigurationSignal::Changed(event))) => {
                                        backoff.reset();
                                        stable = true;
                                        yield ConfigurationWatchEvent::Changed(event);
                                    }
                                    Some(Ok(ConfigurationSignal::Healthy)) => {
                                        if !stable && ready_since.elapsed() >= STABLE_CONNECTION_DURATION {
                                            backoff.reset();
                                            stable = true;
                                        }
                                    }
                                    Some(Err(error)) => break error.to_string(),
                                    None => {
                                        break "firewalld configuration event stream ended".into();
                                    }
                                }
                            }
                        }
                    }
                    Ok(Err(error)) => error.to_string(),
                    Err(_) => "firewalld configuration watcher connection timed out".into(),
                };

                yield ConfigurationWatchEvent::Unavailable(failure);
                sleep(backoff.next_delay()).await;
            }
        })
    }
}

impl RetryBackoff {
    fn new() -> Self {
        Self {
            next_delay: INITIAL_RETRY_DELAY,
        }
    }

    fn next_delay(&mut self) -> Duration {
        let delay = self.next_delay;
        self.next_delay = self.next_delay.saturating_mul(2).min(MAX_RETRY_DELAY);
        delay
    }

    fn reset(&mut self) {
        self.next_delay = INITIAL_RETRY_DELAY;
    }
}
