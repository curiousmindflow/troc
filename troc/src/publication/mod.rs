mod datawriter;
mod publisher;

pub use datawriter::{DataWriter, DataWriterActor, DataWriterActorMessage};
pub use publisher::{Publisher, PublisherActor, PublisherActorCreateObject, PublisherActorMessage};

use troc_core::DurationKind;
use troc_core::{DdsError, ReaderProxy};

#[derive(Debug, Clone)]
pub enum DataWriterEvent {
    SubscriptionMatched(ReaderProxy),
    SubscriptionStopped,
}

#[derive()]
pub struct DataWriterListener {
    receiver: tokio::sync::broadcast::Receiver<DataWriterEvent>,
}

impl DataWriterListener {
    pub(crate) fn new(receiver: tokio::sync::broadcast::Receiver<DataWriterEvent>) -> Self {
        Self { receiver }
    }

    pub async fn wait_event(&mut self) -> Result<DataWriterEvent, DdsError> {
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

    pub async fn wait_subscription_matched(
        &mut self,
        duration: DurationKind,
    ) -> Result<ReaderProxy, DdsError> {
        let fut = async move {
            loop {
                if let DataWriterEvent::SubscriptionMatched(proxy) = self.wait_event().await? {
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
pub struct DataWriterListenerHandle {
    queue_size: usize,
    sender: Option<tokio::sync::broadcast::Sender<DataWriterEvent>>,
}

impl DataWriterListenerHandle {
    pub(crate) fn new(
        queue_size: usize,
        sender: Option<tokio::sync::broadcast::Sender<DataWriterEvent>>,
    ) -> Self {
        Self { queue_size, sender }
    }

    pub async fn send_event(&self, event: DataWriterEvent) {
        if let Some(sender) = &self.sender
            && sender.len() <= self.queue_size
        {
            let _ = sender.send(event);
        }
    }
}
