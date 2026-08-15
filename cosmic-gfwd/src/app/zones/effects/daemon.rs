//! Firewalld daemon-control effects.

use cosmic::Task;

use super::super::Message as ZoneMessage;
use crate::{
    app::{AppModel, Message},
    core::broker::{BrokerError, FirewalldStatus, FwdBroker},
    fl,
};

/// Start or stop firewalld after reserving the global mutation slot.
pub(crate) fn start_firewalld_control(
    app: &mut AppModel,
    start: bool,
) -> Task<cosmic::Action<Message>> {
    let operation = if start {
        fl!("operation-start-firewalld")
    } else {
        fl!("operation-stop-firewalld")
    };
    if !app.begin_mutation(operation) {
        return Task::none();
    }
    app.firewalld_status = FirewalldStatus::Loading;
    Task::perform(control_firewalld(start), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::DaemonControlFinished(result)))
    })
}

/// Start or stop firewalld through the broker.
pub(crate) async fn control_firewalld(start: bool) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    if start {
        broker.start_firewalld().await
    } else {
        broker.stop_firewalld().await
    }
}
