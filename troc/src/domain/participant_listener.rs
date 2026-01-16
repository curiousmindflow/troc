use tokio::sync::broadcast::{Receiver, Sender};
use troc_core::{DdsError, DiscoveredReaderData, DiscoveredWriterData};
use troc_core::{DurationKind, ParticipantProxy};

#[derive(Debug, Clone)]
pub enum ParticipantEvent {
    ParticipantDiscovered { participant_proxy: ParticipantProxy },
    ParticipantUpdated { participant_proxy: ParticipantProxy },
    ParticipantRemoved { participant_proxy: ParticipantProxy },
    ReaderDiscovered { reader_data: DiscoveredReaderData },
    WriterDiscovered { writer_data: DiscoveredWriterData },
}

#[derive()]
pub struct DomainParticipantListener {
    pub(crate) receiver: Receiver<ParticipantEvent>,
}

impl DomainParticipantListener {
    pub async fn wait_event(&mut self) -> Result<ParticipantEvent, DdsError> {
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

    pub async fn wait_participant_discovered(
        &mut self,
        duration: DurationKind,
    ) -> Result<ParticipantProxy, DdsError> {
        let fut = async move {
            loop {
                if let ParticipantEvent::ParticipantDiscovered { participant_proxy } =
                    self.wait_event().await?
                {
                    break Ok(participant_proxy);
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

    pub async fn wait_participant_update(
        &mut self,
        duration: DurationKind,
    ) -> Result<ParticipantProxy, DdsError> {
        let fut = async move {
            loop {
                if let ParticipantEvent::ParticipantUpdated { participant_proxy } =
                    self.wait_event().await?
                {
                    break Ok(participant_proxy);
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

    pub async fn wait_participant_removed(
        &mut self,
        duration: DurationKind,
    ) -> Result<ParticipantProxy, DdsError> {
        let fut = async move {
            loop {
                if let ParticipantEvent::ParticipantRemoved { participant_proxy } =
                    self.wait_event().await?
                {
                    break Ok(participant_proxy);
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

    pub async fn wait_reader_discovered(
        &mut self,
        duration: DurationKind,
    ) -> Result<DiscoveredReaderData, DdsError> {
        let fut = async move {
            loop {
                if let ParticipantEvent::ReaderDiscovered { reader_data } =
                    self.wait_event().await?
                {
                    break Ok(reader_data);
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

    pub async fn wait_writer_discovered(
        &mut self,
        duration: DurationKind,
    ) -> Result<DiscoveredWriterData, DdsError> {
        let fut = async move {
            loop {
                if let ParticipantEvent::WriterDiscovered { writer_data } =
                    self.wait_event().await?
                {
                    break Ok(writer_data);
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

#[derive(Debug, Clone)]
pub struct DomainParticipantListenerHandle {
    queue_size: usize,
    pub(crate) sender: Option<Sender<ParticipantEvent>>,
}

impl DomainParticipantListenerHandle {
    pub fn new(queue_size: usize, sender: Option<Sender<ParticipantEvent>>) -> Self {
        Self { queue_size, sender }
    }

    pub async fn signal_participant_discovered(&self, proxy: ParticipantProxy) {
        self.send_event(ParticipantEvent::ParticipantDiscovered {
            participant_proxy: proxy,
        })
        .await
    }

    pub async fn signal_participant_updated(&self, proxy: ParticipantProxy) {
        self.send_event(ParticipantEvent::ParticipantUpdated {
            participant_proxy: proxy,
        })
        .await
    }

    pub async fn signal_participant_removed(&self, proxy: ParticipantProxy) {
        self.send_event(ParticipantEvent::ParticipantRemoved {
            participant_proxy: proxy,
        })
        .await
    }

    pub async fn signal_reader_discovered(&self, reader_data: DiscoveredReaderData) {
        self.send_event(ParticipantEvent::ReaderDiscovered { reader_data })
            .await
    }

    pub async fn signal_writer_discovered(&self, writer_data: DiscoveredWriterData) {
        self.send_event(ParticipantEvent::WriterDiscovered { writer_data })
            .await
    }

    pub async fn send_event(&self, event: ParticipantEvent) {
        if let Some(sender) = &self.sender
            && sender.len() <= self.queue_size
        {
            let _ = sender.send(event);
        }
    }
}
