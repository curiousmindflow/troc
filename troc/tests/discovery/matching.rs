use troc_core::{QosPolicy, ReliabilityQosPolicy, TopicKind};

use rstest::*;

use crate::{
    discovery::exchange,
    fixture::{TwoParticipantsBundle, build_qos, two_participants},
};

#[rstest]
#[case(ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() }, ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() })]
#[case( ReliabilityQosPolicy::BestEffort, ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() })]
#[case(ReliabilityQosPolicy::BestEffort, ReliabilityQosPolicy::BestEffort)]
#[should_panic]
#[case::should_not_match( ReliabilityQosPolicy::Reliable { max_blocking_time: Default::default() }, ReliabilityQosPolicy::BestEffort)]
#[tokio::test]
async fn reliability(
    _setup_log: (),
    #[case] _reader_reliability_qos: ReliabilityQosPolicy,
    #[case] _writer_reliability_qos: ReliabilityQosPolicy,
    #[with(_reader_reliability_qos)]
    #[from(build_qos)]
    _reader_qos: QosPolicy,
    #[with(_writer_reliability_qos)]
    #[from(build_qos)]
    _writer_qos: QosPolicy,
    #[with(format!("discovery/matching/reliability"), TopicKind::NoKey, _reader_qos, _writer_qos)]
    #[future]
    two_participants: TwoParticipantsBundle,
) {
    let bundle = two_participants.await;
    exchange(bundle, 2).await.unwrap();
}

// #[tokio::test]
// async fn test_reliable_delivery() {
//     // Create two participants
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

//     // Configure for RELIABLE communication
//     let qos = domain_participant_read
//         .create_qos_builder()
//         .history(HistoryQosPolicy::KeepAll)
//         .reliability(ReliabilityQosPolicy::Reliable {
//             max_blocking_time: Default::default(),
//         })
//         .build();

//     let topic = domain_participant_read.create_topic(
//         "/test/matching/test_reliable_delivery",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );

//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();

//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Test 1: Sequential messages should all arrive in order
//     println!("Testing sequential message delivery...");
//     for i in 0..5 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("sequential message {}", i),
//         };
//         data_writer.write(data.clone()).await.unwrap();

//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received message {}: {}", i, message);
//                 assert_eq!(*message, data, "Message content mismatch");
//                 assert_eq!(message.id, i, "Message order incorrect"); // Sequential
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for message {}", i);
//             }
//         }
//     }

//     // Test 2: Burst of messages should all arrive
//     println!("Testing burst message delivery...");
//     let mut sent_messages = Vec::new();
//     for i in 0..5 {
//         println!("Sending burst message {}", i);
//         let data = DummyStruct {
//             id: i,
//             content: format!("burst message {}", i),
//         };
//         sent_messages.push(data.clone());
//         data_writer.write(data).await.unwrap();
//     }

//     // Verify all burst messages are received
//     // for expected_msg in sent_messages.iter().rev() {
//     let mut i: u8 = 0;
//     let mut rcvd_messages = Vec::new();
//     loop {
//         if i == 5 {
//             break;
//         }
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received burst message: {}", message);
//                 rcvd_messages.push(message.clone());
//                 // assert_eq!(*message, expected_msg.clone(), "Burst message content mismatch");
//                 assert_eq!(*message, sent_messages[i as usize], "Burst message content mismatch");

//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for burst message ");
//             }
//         }
//         i += 1;
//     }

//     // }
//     assert_eq!(
//         rcvd_messages.len(),
//         sent_messages.len(),
//         "Not all burst messages received"
//     );
//     println!("Reliability test completed successfully");
// }

// #[tokio::test]
// async fn test_reliable_retransmission_delivery() {
//     // Create writer participant first
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

//     let qos = domain_participant_write
//         .create_qos_builder()
//         .history(HistoryQosPolicy::KeepAll)
//         .reliability(ReliabilityQosPolicy::Reliable {
//             max_blocking_time: Default::default(),
//         })
//         .build();

//     let topic = domain_participant_write.create_topic(
//         "/test/matching/test_reliable_retransmission_delivery",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Write first burst of messages before reader exists
//     println!("Sending first burst of messages...");
//     let mut all_sent_messages = Vec::new();
//     for i in 0..5 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("first burst message {}", i),
//         };
//         println!("Sending message: {}", data);
//         all_sent_messages.push(data.clone());
//         data_writer.write(data).await.unwrap();
//     }

//     // Create reader participant after first burst
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Write second burst of messages
//     println!("Sending second burst of messages...");
//     for i in 5..10 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("second burst message {}", i),
//         };
//         println!("Sending message: {}", data);
//         all_sent_messages.push(data.clone());
//         data_writer.write(data).await.unwrap();
//     }

//     // Read all messages and verify retransmission
//     let mut received_messages = Vec::new();
//     for i in 0..10 {
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received message {}: {}", i, message);
//                 received_messages.push(message.clone());
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for message {}", i);
//             }
//         }
//     }

//     // Verify all messages were received
//     assert_eq!(
//         received_messages.len(),
//         all_sent_messages.len(),
//         "Not all messages were retransmitted and received"
//     );

//     // Verify message order was preserved
//     for (received, sent) in received_messages.iter().zip(all_sent_messages.iter()) {
//         assert_eq!(received, sent, "Message mismatch or wrong order");
//     }

//     println!("Reliability and retransmission test completed successfully");
// }

// #[tokio::test]
// async fn test_best_effort_behavior() {
//     println!("Starting best effort behavior test");

//     // Create two participants
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "alpha").await;
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "beta").await;

//     // Configure for BEST_EFFORT communication
//     let qos = domain_participant_read
//         .create_qos_builder()
//         .reliability(ReliabilityQosPolicy::BestEffort)
//         .build();

//     let topic = domain_participant_read.create_topic(
//         "/test/matching/test_best_effort_behavior",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );

//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();

//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Test 1: Rapid message sending
//     println!("Testing rapid message sending...");
//     for i in 0..10 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("rapid message {}", i),
//         };
//         data_writer.write(data).await.unwrap();
//     }

//     // With best effort, we might not receive all messages
//     // We'll read what we can for a short period
//     let mut received_count = 0;
//     for _ in 0..10 {
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received message: {}", message);
//                 received_count += 1;
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
//                 break;
//             }
//         }
//     }
//     println!("Received {} out of 10 rapid messages", received_count);
//     // With best effort, we don't assert on exact message count

//     // Test 2: Latest value handling
//     println!("Testing latest value handling...");
//     let final_message = DummyStruct {
//         id: 99,
//         content: String::from("final message"),
//     };
//     data_writer.write(final_message.clone()).await.unwrap();

//     // We should be able to read the latest message
//     tokio::select! {
//         read_result = {
//             let future = Box::pin(data_reader.read_next_sample());
//             future
//         } => {
//             let sample = read_result.unwrap();
//             let message = sample.data().unwrap();
//             println!("Received final message: {}", message);
//             assert_eq!(*message, final_message, "Should receive latest message");
//         }
//         _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//             panic!("Timeout waiting for final message");
//         }
//     }

//     println!("Best effort behavior test completed");
// }

// #[tokio::test]
// async fn test_reliable_with_keep_last() {
//     println!("Starting reliable with keep last test");

//     // Create two participants
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "alpha").await;
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "beta").await;

//     // Configure for RELIABLE with KEEP_LAST(3)
//     let qos = domain_participant_read
//         .create_qos_builder()
//         .reliability(ReliabilityQosPolicy::Reliable {
//             max_blocking_time: Default::default(),
//         })
//         .history(HistoryQosPolicy::KeepLast { depth: 3 }) // Limited history
//         .build();

//     let topic = domain_participant_read.create_topic(
//         "/test/matching/test_reliable_with_keep_last",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );

//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();

//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Test 1: Write more messages than history can hold
//     println!("Testing history limit handling...");
//     let mut sent_messages = Vec::new();
//     for i in 0..5 {
//         // Send 5 messages with history of 3
//         let data = DummyStruct {
//             id: i,
//             content: format!("message {}", i),
//         };
//         println!("Writing message: {}", data);
//         data_writer.write(data.clone()).await.unwrap();
//         sent_messages.push(data);
//     }

//     // Should only receive the last 3 messages
//     let mut received_messages = Vec::new();
//     for _ in 0..3 {
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received message: {}", message);
//                 received_messages.push(message.clone());
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for message");
//             }
//         }
//     }

//     // Verify we got the last 3 messages in order
//     assert_eq!(
//         received_messages.len(),
//         3,
//         "Should receive exactly 3 messages"
//     );
//     for (i, msg) in received_messages.iter().enumerate() {
//         assert_eq!(
//             msg.id as usize,
//             i + 2,
//             "Should receive last 3 messages in order"
//         );
//     }

//     // Test 2: Rapid write/read cycle to stress the system
//     println!("Testing under stress...");
//     for i in 0..10 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("stress message {}", i),
//         };
//         data_writer.write(data.clone()).await.unwrap();

//         // Immediately try to read
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received stress message: {}", message);
//                 assert_eq!(*message, data, "Message content should match under stress");
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for stress message {}", i);
//             }
//         }
//     }

//     println!("Reliable with keep last test completed successfully");
// }

// #[tokio::test]
// async fn test_reliable_with_keep_all() {
//     println!("Starting reliable with keep all test");

//     // Create two participants
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "alpha").await;
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "beta").await;

//     // Configure for RELIABLE with KEEP_ALL
//     let qos = domain_participant_read
//         .create_qos_builder()
//         .reliability(ReliabilityQosPolicy::Reliable {
//             max_blocking_time: Default::default(),
//         })
//         .history(HistoryQosPolicy::KeepAll)
//         .build();

//     let topic = domain_participant_read.create_topic(
//         "/test/matching/test_reliable_with_keep_all",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );

//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();

//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Test 1: Write multiple messages and verify all are kept
//     println!("Testing keep all behavior...");
//     let mut sent_messages = Vec::new();
//     for i in 0..10 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("message {}", i),
//         };
//         println!("Writing message: {}", data);
//         data_writer.write(data.clone()).await.unwrap();
//         sent_messages.push(data);
//     }

//     // Simulate slow reader by introducing delay
//     println!("Simulating slow reader...");
//     tokio::time::sleep(std::time::Duration::from_secs(2)).await;

//     // Should receive all messages despite delay
//     let mut received_messages = Vec::new();
//     for i in 0..10 {
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received message {}: {}", i, message);
//                 received_messages.push(message.clone());
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for message {}", i);
//             }
//         }
//     }

//     // Verify all messages were received in order
//     assert_eq!(
//         received_messages.len(),
//         sent_messages.len(),
//         "Should receive all messages"
//     );
//     for (sent, received) in sent_messages.iter().zip(received_messages.iter()) {
//         assert_eq!(sent, received, "Messages should match and be in order");
//     }

//     // Test 2: Continuous write while reader is "busy"
//     println!("Testing continuous write with slow reader...");
//     let mut more_sent_messages = Vec::new();
//     for i in 10..15 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("busy reader message {}", i),
//         };
//         println!("Writing message while reader busy: {}", data);
//         data_writer.write(data.clone()).await.unwrap();
//         more_sent_messages.push(data);
//         // Small delay between writes to simulate continuous publication
//         tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//     }

//     // Reader catches up
//     let mut more_received_messages = Vec::new();
//     for i in 0..5 {
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Slow reader received message {}: {}", i, message);
//                 more_received_messages.push(message.clone());
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for message during catch up {}", i);
//             }
//         }
//     }

//     // Verify catch-up messages
//     assert_eq!(
//         more_received_messages.len(),
//         more_sent_messages.len(),
//         "Should receive all catch-up messages"
//     );
//     for (sent, received) in more_sent_messages.iter().zip(more_received_messages.iter()) {
//         assert_eq!(
//             sent, received,
//             "Catch-up messages should match and be in order"
//         );
//     }

//     println!("Reliable with keep all test completed successfully");
// }

// #[tokio::test]
// async fn test_transient_local_reliable() {
//     println!("Starting transient local reliable test");

//     // Create first participant (writer)
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "writer").await;

//     // Configure for RELIABLE and TRANSIENT_LOCAL
//     let qos = domain_participant_write
//         .create_qos_builder()
//         .reliability(ReliabilityQosPolicy::Reliable {
//             max_blocking_time: Default::default(),
//         })
//         .durability(DurabilityQosPolicy::TransientLocal)
//         .history(HistoryQosPolicy::KeepAll) // Needed for historical data
//         .build();

//     let topic = domain_participant_write.create_topic(
//         "/test/matching/test_transient_local_reliable",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Write historical data before any reader exists
//     println!("Writing historical data...");
//     let mut historical_messages = Vec::new();
//     for i in 0..5 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("historical message {}", i),
//         };
//         println!("Writing historical message: {}", data);
//         data_writer.write(data.clone()).await.unwrap();
//         historical_messages.push(data);
//     }

//     // Create late-joining reader
//     println!("Creating late-joining reader...");
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "reader").await;
//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Verify late joiner receives historical data
//     println!("Verifying historical data delivery...");
//     let mut received_historical = Vec::new();
//     for i in 0..5 {
//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received historical message {}: {}", i, message);
//                 received_historical.push(message.clone());
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for historical message {}", i);
//             }
//         }
//     }

//     // Verify historical messages
//     assert_eq!(
//         received_historical.len(),
//         historical_messages.len(),
//         "Should receive all historical messages"
//     );
//     for (historical, received) in historical_messages.iter().zip(received_historical.iter()) {
//         assert_eq!(
//             historical, received,
//             "Historical messages should match and be in order"
//         );
//     }

//     // Write new data after reader exists
//     println!("Testing new data delivery...");
//     let new_data = DummyStruct {
//         id: 100,
//         content: String::from("new message after reader joined"),
//     };
//     println!("Writing new message: {}", new_data);
//     data_writer.write(new_data.clone()).await.unwrap();

//     // Verify new data is received
//     tokio::select! {
//         read_result = {
//             let future = Box::pin(data_reader.read_next_sample());
//             future
//         } => {
//             let sample = read_result.unwrap();
//             let message = sample.data().unwrap();
//             println!("Received new message: {}", message);
//             assert_eq!(*message, new_data, "New message should match");
//         }
//         _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//             panic!("Timeout waiting for new message");
//         }
//     }

//     println!("Transient local reliable test completed successfully");
// }

// #[tokio::test]
// async fn test_deadline_with_reliability() {
//     println!("Starting deadline with reliability test");

//     // Create two participants
//     let mut domain_participant_read = DomainParticipant::new_with_name(0, "alpha").await;
//     let mut domain_participant_write = DomainParticipant::new_with_name(0, "beta").await;

//     // Configure for RELIABLE with Deadline (1 second)
//     let qos = domain_participant_read
//         .create_qos_builder()
//         .reliability(ReliabilityQosPolicy::Reliable {
//             max_blocking_time: Default::default(),
//         })
//         .deadline(DeadlineQosPolicy {
//             period: Timestamp::default(),
//         })
//         .build();

//     let topic = domain_participant_read.create_topic(
//         "/test/matching/test_deadline_with_reliability",
//         "String",
//         &qos,
//         TopicKind::NoKey,
//     );

//     let mut subscriber = domain_participant_read
//         .create_subscriber(&qos)
//         .await
//         .unwrap();
//     let mut publisher = domain_participant_write
//         .create_publisher(&qos)
//         .await
//         .unwrap();

//     let mut data_reader = subscriber
//         .create_datareader::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();
//     let mut data_writer = publisher
//         .create_datawriter::<DummyStruct>(&topic, &qos)
//         .await
//         .unwrap();

//     // Test 1: Meeting deadline requirements
//     println!("Testing communication within deadline...");
//     for i in 0..3 {
//         let data = DummyStruct {
//             id: i,
//             content: format!("on-time message {}", i),
//         };
//         println!("Writing message {}: {}", i, data);
//         data_writer.write(data.clone()).await.unwrap();

//         tokio::select! {
//             read_result = {
//                 let future = Box::pin(data_reader.read_next_sample());
//                 future
//             } => {
//                 let sample = read_result.unwrap();
//                 let message = sample.data().unwrap();
//                 println!("Received message within deadline: {}", message);
//                 assert_eq!(*message, data);
//             }
//             _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//                 panic!("Timeout waiting for message {}", i);
//             }
//         }

//         // Wait less than deadline before next write
//         tokio::time::sleep(std::time::Duration::from_millis(500)).await;
//     }

//     // Test 2: Missing deadline
//     println!("Testing deadline missed behavior...");
//     let data = DummyStruct {
//         id: 100,
//         content: String::from("delayed message"),
//     };
//     println!("Writing delayed message: {}", data);
//     data_writer.write(data.clone()).await.unwrap();

//     // Wait longer than deadline
//     tokio::time::sleep(std::time::Duration::from_secs(2)).await;

//     // Write another message to verify system still functions
//     let recovery_data = DummyStruct {
//         id: 101,
//         content: String::from("recovery message"),
//     };
//     println!("Writing recovery message: {}", recovery_data);
//     data_writer.write(recovery_data.clone()).await.unwrap();

//     tokio::select! {
//         read_result = {
//             let future = Box::pin(data_reader.read_next_sample());
//             future
//         } => {
//             let sample = read_result.unwrap();
//             let message = sample.data().unwrap();
//             println!("Received recovery message: {}", message);
//             assert_eq!(*message, recovery_data);
//         }
//         _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
//             panic!("Timeout waiting for recovery message");
//         }
//     }

//     println!("Deadline with reliability test completed successfully");
// }
