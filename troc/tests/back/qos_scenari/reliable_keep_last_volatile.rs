use std::time::Duration;

use troc_core::DataReader;
use troc_core::DataWriter;
use troc_core::DurabilityQosPolicy;
use troc_core::HistoryQosPolicy;
use troc_core::Publisher;
use troc_core::ReliabilityQosPolicy;
use troc_core::Subscriber;
use troc_core::Timestamp;
use rstest::rstest;
use troc_core::DomainParticipant;
use crate::fixture::custom_two_participants;
use crate::fixture::mesh;
use crate::fixture::multiple_readers_single_writer;
use crate::fixture::multiple_writers_single_reader;
use crate::fixture::DummyStruct;

const TIMEOUT_TEST: Duration = Duration::from_secs(5);

#[rstest]
#[timeout(TIMEOUT_TEST)]
#[tokio::test]
async fn one_to_one(
    #[future]#[with("/test/reliable_keep_last_volatile/one_to_one", ReliabilityQosPolicy::Reliable { max_blocking_time: Timestamp::default() }, HistoryQosPolicy::KeepLast { depth: 1 }, DurabilityQosPolicy::Volatile)] custom_two_participants: (DataReader<DummyStruct>, DataWriter<DummyStruct>),
) {
    let (mut data_reader, mut data_writer) = custom_two_participants.await;

    let data = DummyStruct {
        id: 0,
        content: String::from("hi"),
    };
    
    data_writer.write(data.clone()).await.unwrap();

    let sample = data_reader.read_next_sample().await.unwrap();
    let message = sample.data().unwrap();
    println!("msg: {}", message);
    assert_eq!(*message, data);
}

#[rstest]
#[tokio::test]
async fn test_many_writers_single_reader(
    #[future] #[with("/test/reliable_keep_last_volatile/test_many_writers_single_reader", 3, ReliabilityQosPolicy::Reliable { max_blocking_time: Timestamp::default() }, HistoryQosPolicy::KeepLast { depth: 1 }, DurabilityQosPolicy::Volatile)] multiple_writers_single_reader: (DataReader<DummyStruct>, Vec<(DomainParticipant, Publisher, DataWriter<DummyStruct>)>),
) {
 let (mut data_reader, mut writers) = multiple_writers_single_reader.await;

 // Each writer sends a message
 for (i, (_participant, _publisher, writer)) in writers.iter_mut().enumerate() {
     let data = DummyStruct {
         id: i as u8,
         content: format!("message from writer {}", i),
     };
     println!("Writer {} sending: {}", i, data);
     writer.write(data.clone()).await.unwrap();

     // Reader should receive the message
     tokio::select! {
         read_result = data_reader.read_next_sample() => {
             let sample = read_result.unwrap();
             let message = sample.data().unwrap();
             println!("Reader received from writer {}: {}", i, message);
             assert_eq!(*message, data);
         }
         _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
             panic!("Timeout waiting for message from writer {}", i);
         }
     }
 }

 println!("Test completed successfully - reader received all messages");
}

#[rstest]
#[timeout(TIMEOUT_TEST)]
#[tokio::test]
async fn test_many_readers_single_writer(
    #[future] #[with("/test/reliable_keep_last_volatile/test_many_readers_single_writer", 3, ReliabilityQosPolicy::Reliable { max_blocking_time: Timestamp::default() }, HistoryQosPolicy::KeepLast { depth: 1 }, DurabilityQosPolicy::Volatile)] multiple_readers_single_writer: (DataWriter<DummyStruct>, Vec<(DomainParticipant, Subscriber, DataReader<DummyStruct>)>),
) {
let (mut data_writer, mut readers) = multiple_readers_single_writer.await;

// Writer sends a message that all readers should receive
let data = DummyStruct {
    id: 42,
    content: String::from("broadcast message"),
};
println!("Writer sending: {}", data);
data_writer.write(data.clone()).await.unwrap();

// Each reader should receive the message
for (i, (_participant, _subscriber, reader)) in readers.iter_mut().enumerate() {
    tokio::select! {
        read_result = reader.read_next_sample() => {
            let sample = read_result.unwrap();
            let message = sample.data().unwrap();
            println!("Reader {} received: {}", i, message);
            assert_eq!(*message, data);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("Timeout waiting for reader {} to receive message", i);
        }
    }
}

println!("Test completed successfully - all readers received the message");
}

#[rstest]
#[timeout(TIMEOUT_TEST)]
#[tokio::test]
async fn test_mesh(
    #[future] #[with("/test/reliable_keep_last_volatile/test_mesh", ReliabilityQosPolicy::Reliable { max_blocking_time: Timestamp::default() }, HistoryQosPolicy::KeepLast { depth: 1 }, DurabilityQosPolicy::Volatile)] mesh: (Vec<(i32, i32, DataWriter<DummyStruct>)>, Vec<(i32, i32, DataReader<DummyStruct>)>),
) {
    let (mut writers, mut readers) = mesh.await;
    
    let data = DummyStruct {
        id: 0,
        content: String::from("hi"),
    };

    // Write data from each writer
    for (from, to, writer) in writers.iter_mut() {
        println!("Participant {} writing to {}", from, to);
        writer.write(data.clone()).await.unwrap();
    }

    // Read and verify data for each reader
    for (to, from, reader) in readers.iter_mut() {
        println!("Participant {} reading from {}", to, from);
        tokio::select! {
            read_result = {
                let future = Box::pin(reader.read_next_sample());
                future
            } => {
                let sample = read_result.unwrap();
                let message = sample.data().unwrap();
                println!("Participant {} received from {}: {}", to, from, message);
                assert_eq!(*message, data, "Message content mismatch");
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("Timeout: participant {} waiting for message from {}", to, from);
            }
        }
    }

    println!("Test completed successfully - all participants communicated in mesh");
}
