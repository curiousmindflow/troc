mod condition;
mod data_sample;
mod datareader;
mod instance_state_kind;
mod sample_info;
mod sample_state_kind;
mod subscriber;
mod view_state_kind;

pub use data_sample::DataSample;
pub use datareader::{
    DataReader, DataReaderActor, DataReaderActorCreateObject, DataReaderActorMessage,
};
pub use subscriber::{
    Subscriber, SubscriberActor, SubscriberActorCreateObject, SubscriberActorMessage,
};

use troc_core::DurationKind;
use troc_core::{DdsError, WriterProxy};

#[derive(Debug, Clone)]
pub enum DataReaderEvent {
    PublicationMatched(WriterProxy),
    PublicationStopped,
}

#[derive()]
pub struct DataReaderListener {
    receiver: tokio::sync::broadcast::Receiver<DataReaderEvent>,
}

impl DataReaderListener {
    pub(crate) fn new(receiver: tokio::sync::broadcast::Receiver<DataReaderEvent>) -> Self {
        Self { receiver }
    }

    pub async fn wait_event(&mut self) -> Result<DataReaderEvent, DdsError> {
        match self.receiver.recv().await {
            Ok(event) => Ok(event),
            Err(e) => match e {
                tokio::sync::broadcast::error::RecvError::Closed => Err(DdsError::OutOfResources),
                tokio::sync::broadcast::error::RecvError::Lagged(l) => {
                    Err(DdsError::Error(format!("Event has been lost, count: {l}")))
                }
            },
        }
    }

    pub async fn wait_publication_matched(
        &mut self,
        duration: DurationKind,
    ) -> Result<WriterProxy, DdsError> {
        let fut = async move {
            loop {
                if let DataReaderEvent::PublicationMatched(proxy) = self.wait_event().await? {
                    break Ok(proxy);
                }
            }
        };

        if let DurationKind::Finite(duration) = duration {
            tokio::time::timeout(duration, fut)
                .await
                .map_err(|e| DdsError::Timeout {
                    cause: e.to_string(),
                })?
        } else {
            fut.await
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataReaderListenerHandle {
    queue_size: usize,
    sender: Option<tokio::sync::broadcast::Sender<DataReaderEvent>>,
}

impl DataReaderListenerHandle {
    pub(crate) fn new(
        queue_size: usize,
        sender: Option<tokio::sync::broadcast::Sender<DataReaderEvent>>,
    ) -> Self {
        Self { queue_size, sender }
    }

    pub async fn send_event(&self, event: DataReaderEvent) {
        if let Some(sender) = &self.sender
            && sender.len() <= self.queue_size
        {
            let _ = sender.send(event);
        }
    }
}
