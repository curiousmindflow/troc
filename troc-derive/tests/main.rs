use troc_core::{KeyCalculationError, Keyed};
use troc_derive::DDSType;

#[test]
fn type_complete() {
    #[derive(Default, DDSType)]
    pub struct Foo {
        #[key]
        var_u8: u8,
        #[key]
        var_u16: u16,
        #[key]
        var_string: String,
        #[key]
        var_bool: bool,
    }

    let dummy = Foo::default();

    let key = dummy.key().unwrap();

    assert_ne!([0u8; 16], key)
}

#[test]
fn foo_1() {
    #[allow(dead_code)]
    #[derive(Default, DDSType)]
    pub struct Foo {
        #[key]
        id: i32,
        x: i32,
        y: i32,
    }

    let foo = Foo {
        id: 0x12345678,
        ..Default::default()
    };

    let key = foo.key().unwrap();

    let expected_key = [
        0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    assert_eq!(key, expected_key)
}

#[test]
fn foo_2() {
    #[allow(dead_code)]
    #[derive(Default, DDSType)]
    pub struct Foo {
        #[key]
        label: String,
        #[key]
        id: i64,
        x: i32,
        y: i32,
    }

    let foo = Foo {
        label: String::from("BLUE"),
        id: 0x123456789abcdef0,
        ..Default::default()
    };

    let key = foo.key().unwrap();

    let expected_key = [
        0xf9, 0x1a, 0x59, 0xe3, 0x2e, 0x45, 0x35, 0xd9, 0xa6, 0x9c, 0xd5, 0xd9, 0xf5, 0xb6, 0xe3,
        0x6e,
    ];

    assert_eq!(key, expected_key)
}
