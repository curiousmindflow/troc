use std::time::Duration;

use tokio::{
    select,
    sync::mpsc::channel,
    time::{Instant, sleep},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct Timer<T> {
    receiver: tokio::sync::mpsc::Receiver<T>,
    sender: tokio::sync::mpsc::Sender<T>,
}

impl<T> Timer<T>
where
    T: Clone + Send + 'static,
{
    pub fn new() -> Self {
        let (sender, receiver) = channel(16);
        Self { receiver, sender }
    }

    pub async fn wait(&mut self) -> T {
        let Some(id) = self.receiver.recv().await else {
            panic!()
        };
        id
    }

    pub fn handle(&self) -> TimerHandle<T> {
        TimerHandle::new(self.sender.clone())
    }
}

#[derive(Debug)]
pub struct TimerHandle<T> {
    cancel_token: CancellationToken,
    sender: tokio::sync::mpsc::Sender<T>,
}

impl<T> TimerHandle<T>
where
    T: Clone + Send + 'static,
{
    fn new(sender: tokio::sync::mpsc::Sender<T>) -> Self {
        let cancel_token = CancellationToken::new();
        Self {
            cancel_token,
            sender,
        }
    }

    pub fn record_periodic_timer(&self, duration: Duration, obj: T) {
        let global_canellation_token = self.cancel_token.child_token();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let sleep = sleep(duration);
            tokio::pin!(sleep);

            loop {
                select! {
                        _ = global_canellation_token.cancelled() => {
                            break
                        }
                        () = &mut sleep => {
                            sleep.as_mut().reset(Instant::now() + duration);
                            if let Err(e) = sender.send(obj.clone()).await {
                                // TODO: trace error event
                                continue;
                            }
                        },
                        else => {
                            //
                            todo!()
                        }
                }
            }
        });
    }

    pub fn record_timer(&self, duration: Duration, obj: T) {
        unimplemented!()
    }
}
