use crate::core::{BrokerError, FwdBroker};

pub(crate) async fn control_firewalld(start: bool) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    if start {
        broker.start_firewalld().await
    } else {
        broker.stop_firewalld().await
    }
}
