mod common;

use crate::common::resources::{
    BenchMessage, create_receiver_udp_socket, create_sender_udp_socket, udp_exchange,
};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let udp_sender = create_sender_udp_socket("localhost:7000").await;
    let udp_receiver = create_receiver_udp_socket("localhost:7000").await;
    let mut bundle = (udp_sender, udp_receiver);

    sleep(Duration::from_secs(2)).await;

    let msg = BenchMessage::new(59 * 1024);

    for _ in 0..100 {
        let msg = msg.clone();

        tokio::time::timeout(Duration::from_millis(100), udp_exchange(&mut bundle, msg))
            .await
            .unwrap();
    }
}
