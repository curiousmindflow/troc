/// This policy is useful for cases where a Topic is expected to have each instance updated periodically. On the publishing side this
/// setting establishes a contract that the application must meet. On the subscribing side the setting establishes a minimum
/// requirement for the remote publishers that are expected to supply the data values.
use std::time::Duration;
// TODO : Wait to deadline to be implemented
use rstest::rstest;
use troc_core::{
    // DeadlineQosPolicy,
    DomainParticipant,
    DurabilityQosPolicy,
    HistoryQosPolicy,
    ReliabilityQosPolicy,
    TopicKind,
};

use crate::fixture::DummyStruct;

/// DataReader expects a new sample
/// updating the value of each instance at least
/// once every deadline period.
#[rstest]
#[tokio::test]
async fn test_endpoint_deadline_finite(// __FIXTURES__
) {
    println!("Starting endpoint deadline test");

    // Create participants with finite deadline
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO : Wait to deadline to be implemented => FINITE
        // .deadline(DeadlineQosPolicy { period: Timestamp::from(Duration::from_secs(1)) }) // 1 second deadline
        .build();

    let topic =
        domain_participant_write.create_topic("/test/deadline", "String", &qos, TopicKind::NoKey);
    let mut publisher = domain_participant_write
        .create_publisher(&qos)
        .await
        .unwrap();
    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Create reader with same deadline
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Test 1: Meeting deadline
    println!("Testing communication within deadline...");
    for i in 0..3 {
        let data = DummyStruct {
            id: i,
            content: format!("deadline message {}", i),
        };
        println!("Writing message within deadline: {}", data);
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = data_reader.read_next_sample() => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message within deadline: {}", message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {}", i);
            }
        }

        // Wait less than deadline before next write
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Test 2: Missing deadline
    println!("Testing deadline missed behavior...");
    let data = DummyStruct {
        id: 100,
        content: String::from("delayed message"),
    };

    // Wait longer than deadline
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("Writing message after deadline: {}", data);
    data_writer.write(data.clone()).await.unwrap();

    // Test 3: Recovery after deadline miss
    println!("Testing recovery after deadline miss...");
    let recovery_data = DummyStruct {
        id: 101,
        content: String::from("recovery message"),
    };
    println!("Writing recovery message: {}", recovery_data);
    data_writer.write(recovery_data.clone()).await.unwrap();

    tokio::select! {
        read_result = data_reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received recovery message: {}", message);
            assert_eq!(*message, recovery_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for recovery message");
        }
    }

    println!("Deadline test completed successfully");
}

/// The default value of the deadline period is infinite.
#[tokio::test]
async fn test_endpoint_deadline_infinite() {
    println!("Starting endpoint infinite deadline test");

    // Create participants with infinite deadline
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO : Wait to deadline to be implemented => INFINITE
        //    .deadline(DeadlineQosPolicy::Infinite)
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/deadline_infinite",
        "String",
        &qos,
        TopicKind::NoKey,
    );
    let mut publisher = domain_participant_write
        .create_publisher(&qos)
        .await
        .unwrap();
    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Create reader with same infinite deadline
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Test 1: Normal communication with varying delays
    println!("Testing communication with varying delays...");
    let delays = [
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(2),
    ];

    for (i, delay) in delays.iter().enumerate() {
        // Wait before sending
        tokio::time::sleep(*delay).await;

        let data = DummyStruct {
            id: i as u8,
            content: format!("message after {} delay", delay.as_millis()),
        };
        println!("Writing message after {:?} delay: {}", delay, data);
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = data_reader.read_next_sample() => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message: {}", message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message after {:?} delay", delay);
            }
        }
    }

    // Test 2: Long pause in communication
    println!("Testing long communication pause...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    let final_data = DummyStruct {
        id: 100,
        content: String::from("message after long pause"),
    };
    println!("Writing message after long pause: {}", final_data);
    data_writer.write(final_data.clone()).await.unwrap();

    tokio::select! {
        read_result = data_reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received message after long pause: {}", message);
            assert_eq!(*message, final_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for message after long pause");
        }
    }

    println!("Infinite deadline test completed successfully");
}

#[tokio::test]
async fn test_late_joiner() {
    println!("Starting late joiner test");

    // Create first participant (writer) with TRANSIENT_LOCAL to keep history
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .history(HistoryQosPolicy::KeepAll)
        .durability(DurabilityQosPolicy::TransientLocal)
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/late_joiner",
        "String",
        &qos,
        TopicKind::NoKey,
    );
    let mut publisher = domain_participant_write
        .create_publisher(&qos)
        .await
        .unwrap();
    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Write historical data before reader exists
    println!("Writing historical data...");
    let mut historical_messages = Vec::new();
    for i in 0..5 {
        let data = DummyStruct {
            id: i,
            content: format!("historical message {}", i),
        };
        println!("Writing historical message: {}", data);
        data_writer.write(data.clone()).await.unwrap();
        historical_messages.push(data);
    }

    // Create late-joining reader
    println!("Creating late-joining reader...");
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Verify late joiner receives historical data
    println!("Verifying historical data delivery...");
    let mut received_historical = Vec::new();
    for i in 0..5 {
        tokio::select! {
            read_result = data_reader.read_next_sample() => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received historical message {}: {}", i, message);
                received_historical.push(message.clone());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for historical message {}", i);
            }
        }
    }

    // Verify historical messages
    assert_eq!(
        received_historical.len(),
        historical_messages.len(),
        "Should receive all historical messages"
    );
    for (historical, received) in historical_messages.iter().zip(received_historical.iter()) {
        assert_eq!(
            historical, received,
            "Historical messages should match and be in order"
        );
    }

    // Write new data after reader exists
    println!("Testing new data delivery...");
    let new_data = DummyStruct {
        id: 100,
        content: String::from("new message after reader joined"),
    };
    println!("Writing new message: {}", new_data);
    data_writer.write(new_data.clone()).await.unwrap();

    // Verify new data is received
    tokio::select! {
        read_result = data_reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received new message: {}", message);
            assert_eq!(*message, new_data, "New message should match");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for new message");
        }
    }

    println!("Late joiner test completed successfully");
}
