use std::{sync::Arc, time::Duration};

use protocol::types::Guid;
use tokio::sync::{
    Notify,
    mpsc::{Receiver, Sender, channel},
};

#[derive(Debug, Clone)]
pub struct ChangeAvailabilityAdapter {
    notifier: Arc<Notify>,
}

impl ChangeAvailabilityAdapter {
    pub fn new() -> Self {
        Self {
            notifier: Arc::new(Notify::new()),
        }
    }

    pub async fn wait(&self) {
        self.notifier.notified().await
    }
}

impl ChangeAvailabilityPort for ChangeAvailabilityAdapter {
    fn on_change_available(&self) {
        self.notifier.notify_one();
    }
}

#[derive(Debug)]
pub struct AckScheduleAdapterBuilder;

impl AckScheduleAdapterBuilder {
    pub fn create_endpoints() -> (AckScheduleSenderAdapter, AckScheduleReceiverAdapter) {
        let (sender, receiver) = channel(128);
        (
            AckScheduleSenderAdapter { sender },
            AckScheduleReceiverAdapter { receiver },
        )
    }
}

#[derive(Debug)]
pub struct AckScheduleReceiverAdapter {
    receiver: Receiver<AckSchedule>,
}

impl AckScheduleReceiverAdapter {
    pub async fn wait_ack_schedule(&mut self) -> Option<AckSchedule> {
        self.receiver.recv().await
    }
}

#[derive(Debug)]
pub struct AckScheduleSenderAdapter {
    sender: Sender<AckSchedule>,
}

impl AckSchedulePort for AckScheduleSenderAdapter {
    fn on_ack_schedule(&self, writer_guid: Guid, delay: Duration) {
        self.sender
            .blocking_send(AckSchedule { writer_guid, delay });
    }
}

#[derive(Debug)]
pub struct AckSchedule {
    pub writer_guid: Guid,
    pub delay: Duration,
}
