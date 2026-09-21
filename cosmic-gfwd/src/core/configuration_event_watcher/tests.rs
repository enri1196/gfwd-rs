use std::{
    collections::VecDeque,
    future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
    time::Duration,
};

use async_stream::stream;
use futures_util::{StreamExt, future::BoxFuture, poll, stream::BoxStream};
use tokio::time::{advance, sleep};

use super::super::{
    broker::{BrokerError, ConfigurationSignal, ConfigurationWatchSession},
    events::ConfigurationEvent,
};
use super::{ConfigurationEventSource, ConfigurationEventWatcher, ConfigurationWatchEvent};

struct ScriptedSource {
    attempts: Arc<AtomicUsize>,
    requested_zones: Arc<Mutex<Vec<Option<String>>>>,
    steps: Arc<Mutex<VecDeque<ConnectStep>>>,
}

enum ConnectStep {
    Fail(&'static str),
    Pending(DropProbe),
    Session(SessionScript),
}

struct SessionScript {
    steps: Vec<SessionStep>,
    drop_probe: DropProbe,
}

enum SessionStep {
    Changed(ConfigurationEvent),
    Healthy,
    HealthyAfter(Duration),
    Error(&'static str),
    Pending,
}

struct DropProbe(Arc<AtomicUsize>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl ScriptedSource {
    fn new(steps: Vec<ConnectStep>) -> (Self, Arc<AtomicUsize>, Arc<Mutex<Vec<Option<String>>>>) {
        let attempts = Arc::new(AtomicUsize::new(0));
        let requested_zones = Arc::new(Mutex::new(Vec::new()));
        let source = Self {
            attempts: Arc::clone(&attempts),
            requested_zones: Arc::clone(&requested_zones),
            steps: Arc::new(Mutex::new(steps.into_iter().collect())),
        };
        (source, attempts, requested_zones)
    }
}

impl ConfigurationEventSource for ScriptedSource {
    fn connect(
        &self,
        selected_zone: Option<String>,
    ) -> BoxFuture<'_, Result<ConfigurationWatchSession, BrokerError>> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        self.requested_zones.lock().unwrap().push(selected_zone);
        let step = self
            .steps
            .lock()
            .unwrap()
            .pop_front()
            .expect("scripted source ran out of connection steps");

        match step {
            ConnectStep::Fail(message) => Box::pin(async move { Err(BrokerError::new(message)) }),
            ConnectStep::Pending(drop_probe) => Box::pin(async move {
                let _drop_probe = drop_probe;
                future::pending::<Result<ConfigurationWatchSession, BrokerError>>().await
            }),
            ConnectStep::Session(script) => Box::pin(async move { Ok(script.into_session()) }),
        }
    }
}

impl SessionScript {
    fn new(steps: Vec<SessionStep>, drop_probe: DropProbe) -> Self {
        Self { steps, drop_probe }
    }

    fn into_session(self) -> ConfigurationWatchSession {
        let signals = Box::pin(stream! {
            let _drop_probe = self.drop_probe;

            for step in self.steps {
                match step {
                    SessionStep::Changed(event) => {
                        yield Ok(ConfigurationSignal::Changed(event));
                    }
                    SessionStep::Healthy => {
                        yield Ok(ConfigurationSignal::Healthy);
                    }
                    SessionStep::HealthyAfter(delay) => {
                        sleep(delay).await;
                        yield Ok(ConfigurationSignal::Healthy);
                    }
                    SessionStep::Error(message) => {
                        yield Err(BrokerError::new(message));
                        return;
                    }
                    SessionStep::Pending => {
                        future::pending::<()>().await;
                    }
                }
            }
        });
        ConfigurationWatchSession { signals }
    }
}

fn drop_probe() -> (DropProbe, Arc<AtomicUsize>) {
    let drops = Arc::new(AtomicUsize::new(0));
    (DropProbe(Arc::clone(&drops)), drops)
}

fn session(steps: Vec<SessionStep>) -> ConnectStep {
    let (drop_probe, _) = drop_probe();
    ConnectStep::Session(SessionScript::new(steps, drop_probe))
}

fn session_with_probe(steps: Vec<SessionStep>) -> (ConnectStep, Arc<AtomicUsize>) {
    let (drop_probe, drops) = drop_probe();
    (
        ConnectStep::Session(SessionScript::new(steps, drop_probe)),
        drops,
    )
}

async fn poll_next(
    stream: &mut BoxStream<'static, ConfigurationWatchEvent>,
) -> Poll<Option<ConfigurationWatchEvent>> {
    let mut next = Box::pin(stream.next());
    poll!(&mut next)
}

async fn next_event(
    stream: &mut BoxStream<'static, ConfigurationWatchEvent>,
) -> ConfigurationWatchEvent {
    stream.next().await.expect("watcher stream ended")
}

fn assert_unavailable(event: ConfigurationWatchEvent, expected: &str) {
    match event {
        ConfigurationWatchEvent::Unavailable(message) => assert_eq!(message, expected),
        other => panic!("expected unavailable event, got {other:?}"),
    }
}

fn assert_ready(event: ConfigurationWatchEvent) {
    assert!(matches!(event, ConfigurationWatchEvent::Ready));
}

async fn next_after_delay(
    stream: &mut BoxStream<'static, ConfigurationWatchEvent>,
    attempts: &AtomicUsize,
    expected_attempts: usize,
    delay: Duration,
) -> ConfigurationWatchEvent {
    let mut next = Box::pin(stream.next());
    assert!(matches!(poll!(&mut next), Poll::Pending));
    advance(delay - Duration::from_millis(1)).await;
    assert!(matches!(poll!(&mut next), Poll::Pending));
    assert_eq!(attempts.load(Ordering::SeqCst), expected_attempts);
    advance(Duration::from_millis(1)).await;
    match poll!(&mut next) {
        Poll::Ready(Some(event)) => event,
        other => panic!("expected an event at the retry deadline, got {other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn initial_connection_failure_retries_then_becomes_ready() {
    let (source, attempts, _) = ScriptedSource::new(vec![
        ConnectStep::Fail("initial connection failed"),
        session(vec![]),
    ]);
    let mut watcher =
        ConfigurationEventWatcher::with_source(Some("public".into()), source).into_stream();

    assert_unavailable(next_event(&mut watcher).await, "initial connection failed");
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    let mut retry = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut retry), Poll::Pending));
    advance(Duration::from_millis(999)).await;
    assert!(matches!(poll!(&mut retry), Poll::Pending));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    advance(Duration::from_millis(1)).await;
    assert_ready(retry.await.expect("ready event"));
}

#[tokio::test(start_paused = true)]
async fn setup_timeout_is_retried() {
    let (drop_probe, drops) = drop_probe();
    let (source, attempts, _) =
        ScriptedSource::new(vec![ConnectStep::Pending(drop_probe), session(vec![])]);
    let mut watcher =
        ConfigurationEventWatcher::with_source(Some("public".into()), source).into_stream();

    let mut pending = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut pending), Poll::Pending));
    advance(Duration::from_secs(15)).await;
    assert_unavailable(
        pending.await.expect("timeout event"),
        "firewalld configuration watcher connection timed out",
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    let mut retry = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut retry), Poll::Pending));
    advance(Duration::from_millis(999)).await;
    assert!(matches!(poll!(&mut retry), Poll::Pending));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    advance(Duration::from_millis(1)).await;
    assert_ready(retry.await.expect("ready event"));
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test(start_paused = true)]
async fn stream_error_drops_session_before_retry() {
    let (first_step, drops) = session_with_probe(vec![SessionStep::Error("stream failed")]);
    let (source, attempts, requested_zones) =
        ScriptedSource::new(vec![first_step, session(vec![])]);
    let mut watcher =
        ConfigurationEventWatcher::with_source(Some("public".into()), source).into_stream();

    assert_ready(next_event(&mut watcher).await);
    assert_unavailable(next_event(&mut watcher).await, "stream failed");
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    assert_ready(next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await);
    assert_eq!(
        *requested_zones.lock().unwrap(),
        vec![Some("public".into()), Some("public".into())]
    );
}

#[tokio::test(start_paused = true)]
async fn clean_eof_is_a_retryable_failure() {
    let (source, attempts, _) = ScriptedSource::new(vec![session(vec![]), session(vec![])]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_ready(next_event(&mut watcher).await);
    assert_unavailable(
        next_event(&mut watcher).await,
        "firewalld configuration event stream ended",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await);
}

#[tokio::test(start_paused = true)]
async fn backoff_doubles_and_caps() {
    let failures = (0..8)
        .map(|_| ConnectStep::Fail("connection failed"))
        .collect();
    let (source, attempts, _) = ScriptedSource::new(failures);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_unavailable(next_event(&mut watcher).await, "connection failed");
    for (expected_attempt, delay) in [1_u64, 2, 4, 8, 16, 30, 30].into_iter().enumerate() {
        let event = next_after_delay(
            &mut watcher,
            &attempts,
            expected_attempt + 1,
            Duration::from_secs(delay),
        )
        .await;
        assert_unavailable(event, "connection failed");
        assert_eq!(attempts.load(Ordering::SeqCst), expected_attempt + 2);
    }
}

#[tokio::test(start_paused = true)]
async fn immediately_ending_sessions_do_not_reset_backoff() {
    let (source, attempts, _) = ScriptedSource::new(vec![
        ConnectStep::Fail("connection failed"),
        session(vec![]),
        session(vec![]),
        session(vec![]),
    ]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_unavailable(next_event(&mut watcher).await, "connection failed");
    assert_ready(next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await);
    assert_unavailable(
        next_event(&mut watcher).await,
        "firewalld configuration event stream ended",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 2, Duration::from_secs(2)).await);
    assert_unavailable(
        next_event(&mut watcher).await,
        "firewalld configuration event stream ended",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 3, Duration::from_secs(4)).await);
}

#[tokio::test(start_paused = true)]
async fn real_event_resets_backoff() {
    let event = ConfigurationEvent::RuntimeZoneChanged {
        zone: "public".into(),
    };
    let (source, attempts, _) = ScriptedSource::new(vec![
        ConnectStep::Fail("first failure"),
        ConnectStep::Fail("second failure"),
        session(vec![
            SessionStep::Changed(event.clone()),
            SessionStep::Error("session failed"),
        ]),
        session(vec![]),
    ]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_unavailable(next_event(&mut watcher).await, "first failure");
    assert_unavailable(
        next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await,
        "second failure",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 2, Duration::from_secs(2)).await);
    match next_event(&mut watcher).await {
        ConfigurationWatchEvent::Changed(ConfigurationEvent::RuntimeZoneChanged { zone }) => {
            assert_eq!(zone, "public")
        }
        other => panic!("expected the scripted configuration event, got {other:?}"),
    }
    assert_unavailable(next_event(&mut watcher).await, "session failed");
    assert_ready(next_after_delay(&mut watcher, &attempts, 3, Duration::from_secs(1)).await);
}

#[tokio::test(start_paused = true)]
async fn healthy_quiet_session_resets_backoff() {
    let (source, attempts, _) = ScriptedSource::new(vec![
        ConnectStep::Fail("first failure"),
        ConnectStep::Fail("second failure"),
        session(vec![
            SessionStep::Healthy,
            SessionStep::HealthyAfter(Duration::from_secs(30)),
            SessionStep::Error("quiet session failed"),
        ]),
        session(vec![]),
    ]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_unavailable(next_event(&mut watcher).await, "first failure");
    assert_unavailable(
        next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await,
        "second failure",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 2, Duration::from_secs(2)).await);

    let mut healthy = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut healthy), Poll::Pending));
    advance(Duration::from_secs(29)).await;
    assert!(matches!(poll!(&mut healthy), Poll::Pending));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    advance(Duration::from_secs(1)).await;
    assert_unavailable(
        healthy.await.expect("failure after the stable marker"),
        "quiet session failed",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 3, Duration::from_secs(1)).await);
}

#[tokio::test(start_paused = true)]
async fn recovery_is_reported_without_configuration_events() {
    let (source, attempts, _) = ScriptedSource::new(vec![
        ConnectStep::Fail("connection failed"),
        session(vec![SessionStep::Pending]),
    ]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_unavailable(next_event(&mut watcher).await, "connection failed");
    assert_ready(next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await);
    assert!(matches!(poll_next(&mut watcher).await, Poll::Pending));
}

#[tokio::test(start_paused = true)]
async fn drop_cancels_pending_connection() {
    let (drop_probe, drops) = drop_probe();
    let (source, attempts, _) = ScriptedSource::new(vec![ConnectStep::Pending(drop_probe)]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    let mut pending = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut pending), Poll::Pending));
    drop(pending);
    drop(watcher);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    advance(Duration::from_secs(60)).await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn drop_cancels_retry_sleep() {
    let (source, attempts, _) = ScriptedSource::new(vec![ConnectStep::Fail("connection failed")]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();
    assert_unavailable(next_event(&mut watcher).await, "connection failed");

    let mut pending = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut pending), Poll::Pending));
    drop(pending);
    drop(watcher);
    advance(Duration::from_secs(60)).await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn drop_cancels_active_session() {
    let (step, drops) = session_with_probe(vec![SessionStep::Pending]);
    let (source, attempts, _) = ScriptedSource::new(vec![step]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();
    assert_ready(next_event(&mut watcher).await);

    let mut pending = Box::pin(watcher.next());
    assert!(matches!(poll!(&mut pending), Poll::Pending));
    drop(pending);
    drop(watcher);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    advance(Duration::from_secs(60)).await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn replacement_zone_owns_a_new_watcher() {
    let (public_probe, public_drops) = drop_probe();
    let (public_source, public_attempts, _) =
        ScriptedSource::new(vec![ConnectStep::Pending(public_probe)]);
    let mut public =
        ConfigurationEventWatcher::with_source(Some("public".into()), public_source).into_stream();
    let mut pending = Box::pin(public.next());
    assert!(matches!(poll!(&mut pending), Poll::Pending));
    drop(pending);
    drop(public);
    assert_eq!(public_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(public_drops.load(Ordering::SeqCst), 1);

    let (work_probe, work_drops) = drop_probe();
    let (work_source, work_attempts, requested_zones) =
        ScriptedSource::new(vec![ConnectStep::Pending(work_probe)]);
    let mut work =
        ConfigurationEventWatcher::with_source(Some("work".into()), work_source).into_stream();
    let mut pending = Box::pin(work.next());
    assert!(matches!(poll!(&mut pending), Poll::Pending));
    assert_eq!(work_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(*requested_zones.lock().unwrap(), vec![Some("work".into())]);
    drop(pending);
    drop(work);
    assert_eq!(work_drops.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn no_selected_zone_is_forwarded() {
    let (source, attempts, requested_zones) =
        ScriptedSource::new(vec![session(vec![]), session(vec![])]);
    let mut watcher = ConfigurationEventWatcher::with_source(None, source).into_stream();

    assert_ready(next_event(&mut watcher).await);
    assert_unavailable(
        next_event(&mut watcher).await,
        "firewalld configuration event stream ended",
    );
    assert_ready(next_after_delay(&mut watcher, &attempts, 1, Duration::from_secs(1)).await);
    assert_eq!(*requested_zones.lock().unwrap(), vec![None, None]);
}
