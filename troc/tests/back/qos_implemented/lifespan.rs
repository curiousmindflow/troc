use std::time::Duration;

use troc_core::{DomainParticipant, ReliabilityQosPolicy, TopicKind};

use crate::fixture::DummyStruct;

/// Specifies the maximum duration of validity
/// of the data written by the DataWriter The
/// default value of the lifespan duration is
/// infinite.

#[tokio::test]
async fn test_endpoint_lifespan_finite() {
    println!("Starting endpoint finite lifespan test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with finite lifespan (1 second)
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO : Wait to lifespan to be implemented => FINITE
        // .deadline(Lifespan { period: Timestamp::from(Duration::from_secs(1)) }) // 1 second deadline
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/lifespan_finite",
        "String",
        &qos,
        TopicKind::NoKey,
    );

    let mut publisher = domain_participant_write
        .create_publisher(&qos)
        .await
        .unwrap();
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();

    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Test 1: Message received before lifespan expiration
    let data1 = DummyStruct {
        id: 1,
        content: String::from("message before expiration"),
    };
    println!("Writing message to be read before lifespan expiration");
    data_writer.write(data1.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received message before expiration: {}", message);
            assert_eq!(*message, data1);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for message before expiration");
        }
    }

    // Test 2: Message should expire
    let data2 = DummyStruct {
        id: 2,
        content: String::from("message to expire"),
    };
    println!("Writing message that should expire");
    data_writer.write(data2.clone()).await.unwrap();

    // Wait for lifespan to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test 3: Write new message after expiration
    let data3 = DummyStruct {
        id: 3,
        content: String::from("message after expiration"),
    };
    println!("Writing message after previous message expiration");
    data_writer.write(data3.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received new message: {}", message);
            assert_eq!(*message, data3, "Should receive new message, not expired one");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for message after expiration");
        }
    }

    println!("Finite lifespan test completed successfully");
}

#[tokio::test]
async fn test_endpoint_lifespan_infinite() {
    println!("Starting endpoint infinite lifespan test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with infinite lifespan
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        //    TODO : Wait to lifespan to be implemented => INFINITE
        //    .lifespan(LifespanQosPolicy::Infinite)
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/lifespan_infinite",
        "String",
        &qos,
        TopicKind::NoKey,
    );

    let mut publisher = domain_participant_write
        .create_publisher(&qos)
        .await
        .unwrap();
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();

    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Test 1: Message received after short delay
    let data1 = DummyStruct {
        id: 1,
        content: String::from("immediate message"),
    };
    println!("Writing first message");
    data_writer.write(data1.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received immediate message: {}", message);
            assert_eq!(*message, data1);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for immediate message");
        }
    }

    // Test 2: Message received after longer delay
    tokio::time::sleep(Duration::from_secs(2)).await;
    let data2 = DummyStruct {
        id: 2,
        content: String::from("delayed message"),
    };
    println!("Writing message after delay");
    data_writer.write(data2.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received delayed message: {}", message);
            assert_eq!(*message, data2);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for delayed message");
        }
    }

    // Test 3: Multiple messages with varying delays
    for i in 0..3 {
        tokio::time::sleep(Duration::from_secs((i as u8).into())).await; // Delay 0, 1, 2 seconds
        let data = DummyStruct {
            id: 10 + i,
            content: format!("sequential message {}", i),
        };
        println!("Writing sequential message {}", i);
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received sequential message {}: {}", i, message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for sequential message {}", i);
            }
        }
    }

    println!("Infinite lifespan test completed successfully");
}
