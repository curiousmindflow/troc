use std::time::Duration;

use crate::fixture::DummyStruct;
use troc_core::DomainParticipant;
use troc_core::DurabilityQosPolicy;
use troc_core::HistoryQosPolicy;
use troc_core::ReliabilityQosPolicy;
use troc_core::TopicKind;

#[tokio::test]
async fn test_topic_lifecycle() {
    // Create two participants
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "alpha").await;
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "beta").await;

    let qos = domain_participant_read
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .build();

    // Create topics in both participants
    println!("Creating topics...");
    let topic_read = domain_participant_read.create_topic(
        "/test/communication/qos_basic/test_topic_lifecycle",
        "String",
        &qos,
        TopicKind::NoKey,
    );
    let topic_write = domain_participant_write.create_topic(
        "/test/communication/qos_basic/test_topic_lifecycle",
        "String",
        &qos,
        TopicKind::NoKey,
    );

    // Verify topic information
    assert_eq!(topic_read.topic_name(), "/test");
    assert_eq!(topic_read.type_name(), "String");
    assert_eq!(topic_write.topic_name(), "/test");
    assert_eq!(topic_write.type_name(), "String");

    // Create reader and writer
    let mut subscriber = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic_read, &qos)
        .await
        .unwrap();

    let mut publisher = domain_participant_write
        .create_publisher(&qos)
        .await
        .unwrap();
    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic_write, &qos)
        .await
        .unwrap();

    // Verify communication works
    let data = DummyStruct {
        id: 1,
        content: String::from("topic test"),
    };
    println!("Writing to topic...");
    data_writer.write(data.clone()).await.unwrap();

    tokio::select! {
        read_result = data_reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Message received on topic: {}", message);
            assert_eq!(*message, data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for topic data");
        }
    }

    // Drop reader and writer
    println!("Dropping reader and writer...");
    drop(data_reader);
    drop(data_writer);

    // Create new reader and writer on same topics
    println!("Creating new reader and writer on same topics...");
    let mut data_reader = subscriber
        .create_datareader::<DummyStruct>(&topic_read, &qos)
        .await
        .unwrap();
    let mut data_writer = publisher
        .create_datawriter::<DummyStruct>(&topic_write, &qos)
        .await
        .unwrap();

    // Verify communication still works
    let data = DummyStruct {
        id: 2,
        content: String::from("topic reuse test"),
    };
    println!("Writing to reused topic...");
    data_writer.write(data.clone()).await.unwrap();

    tokio::select! {
        read_result = data_reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Message received on reused topic: {}", message);
            assert_eq!(*message, data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for reused topic data");
        }
    }
}

#[tokio::test]
async fn test_deadline_enforcement() {
    println!("Starting deadline enforcement test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with a 1-second deadline
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO : Wait to deadline to be implemented => FINITE
        // .deadline(DeadlineQosPolicy { period: Timestamp::from(Duration::from_secs(1)) }) // 1 second deadline
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/deadline_enforcement",
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

    // Test 1: Meeting deadline
    println!("Testing messages within deadline...");
    for i in 0..3 {
        let data = DummyStruct {
            id: i,
            content: format!("on-time message {}", i),
        };
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message within deadline: {}", message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {}", i);
            }
        }

        // Wait less than deadline before next message
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Test 2: Missing deadline
    println!("Testing deadline missed behavior...");
    // Wait longer than deadline
    tokio::time::sleep(Duration::from_secs(2)).await;

    let late_data = DummyStruct {
        id: 100,
        content: String::from("late message"),
    };
    data_writer.write(late_data.clone()).await.unwrap();

    // Test 3: Recovery after missed deadline
    println!("Testing recovery after missed deadline...");
    let recovery_data = DummyStruct {
        id: 101,
        content: String::from("recovery message"),
    };
    data_writer.write(recovery_data.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received recovery message: {}", message);
            assert_eq!(*message, recovery_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for recovery message");
        }
    }

    println!("Deadline enforcement test completed successfully");
}

#[tokio::test]
async fn test_automatic_liveliness() {
    println!("Starting automatic liveliness test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with automatic liveliness and 1-second lease duration
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO : Wait Liveliness to be implemented => AUTOMATIC
        // .liveliness(LivelinessQosPolicy::Automatic {
        //     lease_duration: Duration::from_secs(1),
        // })
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/automatic_liveliness",
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

    // Test 1: Normal automatic liveliness behavior
    println!("Testing normal automatic liveliness...");
    for i in 0..3 {
        let data = DummyStruct {
            id: i,
            content: format!("automatic liveliness message {}", i),
        };
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message {}: {}", i, message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {}", i);
            }
        }

        // Wait less than lease duration
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Test 2: Extended silence but within lease duration
    println!("Testing extended silence within lease duration...");
    tokio::time::sleep(Duration::from_millis(750)).await;

    let data = DummyStruct {
        id: 100,
        content: String::from("after silence message"),
    };
    data_writer.write(data.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received message after silence: {}", message);
            assert_eq!(*message, data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for message after silence");
        }
    }

    // Test 3: Continuous communication maintains liveliness
    println!("Testing continuous communication...");
    for i in 0..3 {
        let data = DummyStruct {
            id: 200 + i,
            content: format!("continuous message {}", i),
        };
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received continuous message {}: {}", i, message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for continuous message {}", i);
            }
        }

        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("Automatic liveliness test completed successfully");
}

#[tokio::test]
async fn test_manual_liveliness() {
    println!("Starting manual liveliness test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with manual liveliness and 1-second lease duration
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        // TODO : Wait Liveliness to be implemented => MANUAL
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/manual_liveliness",
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

    // Test 1: Communication with manual liveliness assertions
    println!("Testing manual liveliness assertions...");
    for i in 0..3 {
        // Assert liveliness before sending
        // TODO: Wait to be implemented
        //    domain_participant_write.assert_liveliness().await;

        let data = DummyStruct {
            id: i,
            content: format!("manual liveliness message {}", i),
        };
        data_writer.write(data.clone()).await.unwrap();

        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received message after assertion: {}", message);
                assert_eq!(*message, data);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {}", i);
            }
        }

        // Wait but less than lease duration
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Test 2: Missed liveliness assertion
    println!("Testing missed liveliness assertion...");
    // Wait longer than lease duration without asserting liveliness
    tokio::time::sleep(Duration::from_secs(2)).await;

    let data = DummyStruct {
        id: 100,
        content: String::from("message without assertion"),
    };
    data_writer.write(data.clone()).await.unwrap();

    // Test 3: Recovery with new assertion
    println!("Testing recovery with new assertion...");
    // TODO: Wait to be implemented
    //    domain_participant_write.assert_liveliness().await;

    let recovery_data = DummyStruct {
        id: 101,
        content: String::from("recovery message"),
    };
    data_writer.write(recovery_data.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received recovery message: {}", message);
            assert_eq!(*message, recovery_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for recovery message");
        }
    }

    // Test 4: Rapid assertions
    // What happens if we assert liveliness too quickly?
    println!("Testing rapid assertions...");
    for _i in 0..3 {
        // TODO: Wait to be implemented
        //    domain_participant_write.assert_liveliness().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let final_data = DummyStruct {
        id: 200,
        content: String::from("final message"),
    };
    data_writer.write(final_data.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received final message: {}", message);
            assert_eq!(*message, final_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for final message");
        }
    }

    println!("Manual liveliness test completed successfully");
}

#[tokio::test]
async fn test_volatile_durability() {
    println!("Starting volatile durability test");

    // Create first participant (writer)
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .durability(DurabilityQosPolicy::Volatile)
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/volatile_durability",
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

    // Write some data before any reader exists
    println!("Writing data before reader exists...");
    let historical_data = DummyStruct {
        id: 1,
        content: String::from("historical message"),
    };
    data_writer.write(historical_data.clone()).await.unwrap();

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

    // Short delay to ensure discovery is complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Write new data after reader exists
    println!("Writing new data after reader exists...");
    let new_data = DummyStruct {
        id: 2,
        content: String::from("new message"),
    };
    data_writer.write(new_data.clone()).await.unwrap();

    // Late joiner should only receive new data
    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Late joiner received message: {}", message);
            assert_eq!(*message, new_data, "Late joiner should only receive new data, not historical data");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for new message");
        }
    }

    // Verify no more messages are available (historical data should not be received)
    println!("Verifying no historical data is available...");
    tokio::select! {
        _ = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            panic!("Should not receive historical data with VOLATILE durability");
        }
        _ = tokio::time::sleep(Duration::from_secs(1)) => {
            println!("Correctly received no historical data");
        }
    }

    println!("Volatile durability test completed successfully");
}

#[tokio::test]
async fn test_transient_local_durability() {
    println!("Starting transient local durability test");

    // Create first participant (writer)
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .durability(DurabilityQosPolicy::TransientLocal)
        .history(HistoryQosPolicy::KeepAll) // Need KeepAll to maintain history
        .build();

    let topic = domain_participant_write.create_topic(
        "/test/transient_local_durability",
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

    // Write multiple samples before any reader exists
    println!("Writing historical data...");
    let mut historical_messages = Vec::new();
    for i in 0..3 {
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
    for (i, expected_data) in historical_messages.iter().enumerate() {
        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received historical message {}: {}", i, message);
                assert_eq!(*message, *expected_data, "Historical message mismatch");
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for historical message {}", i);
            }
        }
    }

    // Write new data after reader exists
    println!("Writing new data...");
    let new_data = DummyStruct {
        id: 100,
        content: String::from("new message"),
    };
    data_writer.write(new_data.clone()).await.unwrap();

    // Verify reader receives new data
    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received new message: {}", message);
            assert_eq!(*message, new_data, "New message mismatch");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for new message");
        }
    }

    // Create another late joiner to verify persistence
    println!("Creating second late-joining reader...");
    let mut subscriber2 = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader2 = subscriber2
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Verify second late joiner receives all data
    println!("Verifying persistence for second reader...");
    let all_messages = historical_messages.iter().chain(std::iter::once(&new_data));
    for (i, expected_data) in all_messages.enumerate() {
        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader2.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Second reader received message {}: {}", i, message);
                assert_eq!(*message, *expected_data, "Message mismatch for second reader");
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for message {} for second reader", i);
            }
        }
    }

    println!("Transient local durability test completed successfully");
}

#[tokio::test]
async fn test_keep_last_behavior() {
    println!("Starting keep last history test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with KEEP_LAST(3) history
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .history(HistoryQosPolicy::KeepLast { depth: 3 })
        .build();

    let topic =
        domain_participant_write.create_topic("/test/keep_last", "String", &qos, TopicKind::NoKey);

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

    // Test 1: Write more samples than history can hold
    println!("Testing history limit...");
    for i in 0..5 {
        // Write 5 samples with history of 3
        let data = DummyStruct {
            id: i,
            content: format!("sample {}", i),
        };
        println!("Writing sample: {}", data);
        data_writer.write(data.clone()).await.unwrap();
    }

    // Should only receive last 3 samples (2, 3, 4)
    let mut received_samples = Vec::new();
    for _ in 0..3 {
        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received sample: {}", message);
                received_samples.push(message.clone());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for sample");
            }
        }
    }

    // Verify we got the last 3 samples in order
    assert_eq!(
        received_samples.len(),
        3,
        "Should receive exactly 3 samples"
    );
    assert_eq!(
        received_samples[0].id, 2,
        "First received sample should be id 2"
    );
    assert_eq!(
        received_samples[2].id, 4,
        "Last received sample should be id 4"
    );

    // Test 2: Sample replacement
    println!("Testing sample replacement...");
    let replacement_data = DummyStruct {
        id: 100,
        content: String::from("replacement sample"),
    };
    data_writer.write(replacement_data.clone()).await.unwrap();

    tokio::select! {
        read_result = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Received replacement sample: {}", message);
            assert_eq!(*message, replacement_data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for replacement sample");
        }
    }

    // Test 3: Verify no more samples available
    println!("Verifying no more samples...");
    tokio::select! {
        _ = {
            let future = Box::pin(data_reader.read_next_sample());
            future
        } => {
            panic!("Should not receive more samples");
        }
        _ = tokio::time::sleep(Duration::from_secs(1)) => {
            println!("Correctly received no more samples");
        }
    }

    println!("Keep last history test completed successfully");
}

#[tokio::test]
async fn test_keep_all_behavior() {
    println!("Starting keep all history test");

    // Create participants
    let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;
    let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;

    // Configure QoS with KEEP_ALL history
    let qos = domain_participant_write
        .create_qos_builder()
        .reliability(ReliabilityQosPolicy::Reliable {
            max_blocking_time: Default::default(),
        })
        .history(HistoryQosPolicy::KeepAll)
        .build();

    let topic =
        domain_participant_write.create_topic("/test/keep_all", "String", &qos, TopicKind::NoKey);

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

    // Test 1: Write multiple samples
    println!("Testing unlimited history...");
    let mut written_samples = Vec::new();
    for i in 0..10 {
        let data = DummyStruct {
            id: i,
            content: format!("sample {}", i),
        };
        println!("Writing sample: {}", data);
        data_writer.write(data.clone()).await.unwrap();
        written_samples.push(data);
    }

    // Should receive all samples
    println!("Reading all samples...");
    let mut received_samples = Vec::new();
    for i in 0..10 {
        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Received sample {}: {}", i, message);
                received_samples.push(message.clone());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for sample {}", i);
            }
        }
    }

    // Verify all samples received in order
    assert_eq!(
        received_samples.len(),
        written_samples.len(),
        "Should receive all samples"
    );
    for (received, written) in received_samples.iter().zip(written_samples.iter()) {
        assert_eq!(received, written, "Samples should match and be in order");
    }

    // Test 2: Late joiner gets all history
    println!("Testing late joiner...");
    let mut subscriber2 = domain_participant_read
        .create_subscriber(&qos)
        .await
        .unwrap();
    let mut data_reader2 = subscriber2
        .create_datareader::<DummyStruct>(&topic, &qos)
        .await
        .unwrap();

    // Late joiner should receive all previous samples
    let mut late_received_samples = Vec::new();
    for i in 0..10 {
        tokio::select! {
            read_result = {
                let future = Box::pin(data_reader2.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Late joiner received sample {}: {}", i, message);
                late_received_samples.push(message.clone());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout waiting for late joiner sample {}", i);
            }
        }
    }

    // Verify late joiner received all samples
    assert_eq!(
        late_received_samples, written_samples,
        "Late joiner should receive all samples"
    );

    // Test 3: Continue writing after history built up
    println!("Testing additional writes...");
    for i in 10..15 {
        let data = DummyStruct {
            id: i,
            content: format!("additional sample {}", i),
        };
        println!("Writing additional sample: {}", data);
        data_writer.write(data.clone()).await.unwrap();
        written_samples.push(data);

        // Verify both readers receive new samples
        for reader_num in 0..2 {
            let reader = if reader_num == 0 {
                &mut data_reader
            } else {
                &mut data_reader2
            };
            tokio::select! {
                read_result = {
                    let future = Box::pin(reader.read_next_sample());
                    future
                } => {
                    let sample = read_result.unwrap();
                    let message = sample.data().unwrap();
                    println!("Reader {} received new sample: {}", reader_num, message);
                    assert_eq!(*message, written_samples.last().unwrap().clone());
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    panic!("Timeout waiting for new sample on reader {}", reader_num);
                }
            }
        }
    }

    println!("Keep all history test completed successfully");
}
