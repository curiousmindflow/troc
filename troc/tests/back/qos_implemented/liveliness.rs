// TODO import liveliness QOS
// use common::types::LivelinessQosPolicy;
use troc_core::{DomainParticipant, ReliabilityQosPolicy, TopicKind};

use crate::fixture::DummyStruct;

/// The Service will assume that as long as at
/// least one Entity within the
/// DomainParticipant has asserted its
/// liveliness the other Entities in that same
/// DomainParticipant are also alive.
#[tokio::test]
async fn test_endpoint_liveliness_manual() {
    println!("Starting endpoint manual liveliness test");

    // Create participants with manual liveliness
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO Wait to liveliness to be implemented : ManualByParticipant
        //    .liveness(LivelinessQosPolicy { kind:common::types::LivelinessKind::ManualByParticipant, lease_duration: std::time::Duration::from_secs(1) })
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/liveliness_manual",
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

    // Create reader with same liveliness settings
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Test 1: Basic communication with manual liveliness assertions
    println!("Testing communication with manual liveliness assertions...");
    for i in 0..3 {
        // Assert liveliness before sending
        // TODO Having a mechanism that assert liveliness .assert_liviliness()

        //    2.2.2.4.2.22 assert_liveliness
        //    This operation manually asserts the liveliness of the DataWriter. This is used in combination with the LIVELINESS QoS
        //    policy (see 2.2.3, Supported QoS) to indicate to the Service that the entity remains active.
        //    This operation need only be used if the LIVELINESS setting is either MANUAL_BY_PARTICIPANT or
        //    MANUAL_BY_TOPIC. Otherwise, it has no effect.

        let data = DummyStruct {
            id: i,
            content: format!("manual liveliness message {}", i),
        };
        println!("Writing message after liveliness assertion: {}", data);
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = data_reader.read_next_sample() => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message: {}", message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {}", i);
            }
        }

        // Wait but less than lease duration
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Test 2: Missing liveliness assertion
    println!("Testing missed liveliness assertion...");
    // Wait longer than lease duration without asserting liveliness
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let data = DummyStruct {
        id: 100,
        content: String::from("message after missed assertion"),
    };
    println!("Writing message without liveliness assertion: {}", data);
    data_writer.write(data.clone()).await.unwrap();

    // Test 3: Recovery with new liveliness assertion
    println!("Testing recovery with new liveliness assertion...");
    // TODO assert liveliness

    let recovery_data = DummyStruct {
        id: 101,
        content: String::from("recovery message"),
    };
    println!(
        "Writing recovery message after new assertion: {}",
        recovery_data
    );
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

    println!("Manual liveliness test completed successfully");
}

/// The Service will only assume liveliness of
/// the DataWriter if the application has
/// asserted liveliness of that DataWriter itself.
#[tokio::test]
async fn test_endpoint_liveliness_by_topic() {
    println!("Starting endpoint by-topic liveliness test");

    // Create participants with by-topic liveliness
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO Wait to liveliness to be implemented : ManualByTopic
        //    .liveness(LivelinessQosPolicy { kind:common::types::LivelinessKind::ManualByTopic, lease_duration: std::time::Duration::from_secs(1) })
        //    })
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/liveliness_by_topic",
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

    // Create reader with same liveliness settings
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Test 1: Basic communication with by-topic liveliness
    println!("Testing communication with by-topic liveliness assertions...");
    for i in 0..3 {
        // Assert liveliness before sending
        // TODO Having a mechanism that assert liveliness .assert_liviliness()

        let data = DummyStruct {
            id: i,
            content: format!("by-topic liveliness message {}", i),
        };
        println!("Writing message after topic liveliness assertion: {}", data);
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = data_reader.read_next_sample() => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message: {}", message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {}", i);
            }
        }

        // Wait but less than lease duration
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Test 2: Missing topic liveliness assertion
    println!("Testing missed topic liveliness assertion...");
    // Wait longer than lease duration without asserting liveliness
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let data = DummyStruct {
        id: 100,
        content: String::from("message after missed topic assertion"),
    };
    println!(
        "Writing message without topic liveliness assertion: {}",
        data
    );
    data_writer.write(data.clone()).await.unwrap();

    // Test 3: Recovery with new topic liveliness assertion
    println!("Testing recovery with new topic liveliness assertion...");
    // Assert liveliness before sending
    // TODO Having a mechanism that assert liveliness .assert_liviliness()

    let recovery_data = DummyStruct {
        id: 101,
        content: String::from("topic recovery message"),
    };
    println!(
        "Writing recovery message after new topic assertion: {}",
        recovery_data
    );
    data_writer.write(recovery_data.clone()).await.unwrap();

    tokio::select! {
        read_result = data_reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received topic recovery message: {}", message);
            assert_eq!(*message, recovery_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for topic recovery message");
        }
    }

    println!("By-topic liveliness test completed successfully");
}
