use bytes::BytesMut;
use troc_core::{ContentNature, MessageFactory, SerializedData};

fn main() {
    let msg = MessageFactory::new(Default::default())
        .message()
        .reader(Default::default())
        .writer(Default::default())
        .data(
            ContentNature::Data,
            Default::default(),
            None,
            Some(SerializedData::from_slice(
                &std::iter::repeat_n(7u8, 59 * 1024).collect::<Vec<u8>>(),
            )),
        )
        .build();

    let mut buffer = BytesMut::zeroed(65 * 1024);

    for _ in 0..10000 {
        msg.serialize_to(&mut buffer).unwrap();
    }
}
