use std::time::Duration;

use protocol::NackedDataSchedulePort;
use protocol::types::Guid;
use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug)]
pub struct NackedDataScheduleAdapterBuilder;

impl NackedDataScheduleAdapterBuilder {
    pub fn create_endpoints() -> (
        NackedDataScheduleSenderAdapter,
        NackedDataScheduleReceiverAdapter,
    ) {
        let (sender, receiver) = channel(128);
        (
            NackedDataScheduleSenderAdapter { sender },
            NackedDataScheduleReceiverAdapter { receiver },
        )
    }
}

#[derive(Debug)]
pub struct NackedDataScheduleReceiverAdapter {
    receiver: Receiver<NackedSchedule>,
}

impl NackedDataScheduleReceiverAdapter {
    pub async fn wait_nacked_schedule(&mut self) -> Option<NackedSchedule> {
        self.receiver.recv().await
    }
}

#[derive(Debug)]
pub struct NackedDataScheduleSenderAdapter {
    sender: Sender<NackedSchedule>,
}

impl NackedDataSchedulePort for NackedDataScheduleSenderAdapter {
    fn on_nacked_data_schedule(&self, reader_guid: Guid, delay: Duration) {
        let _ = self
            .sender
            .blocking_send(NackedSchedule { reader_guid, delay });
    }
}

#[derive(Debug)]
pub struct NackedSchedule {
    pub reader_guid: Guid,
    pub delay: Duration,
}
