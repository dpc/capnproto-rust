/*
 * Copyright (c) 2013 - 2014, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

#![crate_type = "bin"]

extern crate capnp;

pub mod test_capnp;

mod tests {
    use std;
    use capnp::message::{MessageBuilder, MallocMessageBuilder, SUGGESTED_ALLOCATION_STRATEGY};

    #[test]
    fn test_prim_list () {

        use test_capnp::TestPrimList;

        // Make the first segment small to force allocation of a second segment.
        let mut message = MallocMessageBuilder::new(50, SUGGESTED_ALLOCATION_STRATEGY);

        let testPrimList = message.init_root::<TestPrimList::Builder>();

        let uint8_list = testPrimList.init_uint8_list(100);

        for i in range(0, uint8_list.size()) {
            uint8_list.set(i, i as u8);
        }

        let uint64List = testPrimList.init_uint64_list(20);

        for i in range(0, uint64List.size()) {
            uint64List.set(i, i as u64);
        }

        assert_eq!(testPrimList.has_bool_list(), false);
        let boolList = testPrimList.init_bool_list(65);
        assert_eq!(testPrimList.has_bool_list(), true);

        boolList.set(0, true);
        boolList.set(1, true);
        boolList.set(2, true);
        boolList.set(3, true);
        boolList.set(5, true);
        boolList.set(8, true);
        boolList.set(13, true);
        boolList.set(64, true);

        assert!(boolList[0]);
        assert!(!boolList[4]);
        assert!(!boolList[63]);
        assert!(boolList[64]);

        assert_eq!(testPrimList.has_void_list(), false);
        let voidList = testPrimList.init_void_list(1025);
        assert_eq!(testPrimList.has_void_list(), true);
        voidList.set(257, ());


        let testPrimListReader = testPrimList.as_reader();
        let uint8List = testPrimListReader.get_uint8_list().unwrap();
        for i in range(0, uint8List.size()) {
            assert_eq!(uint8List[i], i as u8);
        }
        let uint64List = testPrimListReader.get_uint64_list().unwrap();
        for i in range(0, uint64List.size()) {
            assert_eq!(uint64List[i], i as u64);
        }

        assert_eq!(testPrimListReader.has_bool_list(), true);
        let boolList = testPrimListReader.get_bool_list().unwrap();
        assert!(boolList[0]);
        assert!(boolList[1]);
        assert!(boolList[2]);
        assert!(boolList[3]);
        assert!(!boolList[4]);
        assert!(boolList[5]);
        assert!(!boolList[6]);
        assert!(!boolList[7]);
        assert!(boolList[8]);
        assert!(!boolList[9]);
        assert!(!boolList[10]);
        assert!(!boolList[11]);
        assert!(!boolList[12]);
        assert!(boolList[13]);
        assert!(!boolList[63]);
        assert!(boolList[64]);

        assert_eq!(testPrimListReader.get_void_list().unwrap().size(), 1025);
    }

    #[test]
    fn test_struct_list () {

        use test_capnp::TestStructList;

        let mut message = MallocMessageBuilder::new_default();

        let test_struct_list = message.init_root::<TestStructList::Builder>();

        test_struct_list.init_struct_list(4);
        let struct_list = test_struct_list.get_struct_list().unwrap();
        struct_list[0].init_uint8_list(1).set(0, 5u8);

        // why does the next line pass the typechecker?
//        struct_list[0].get_uint8_list()[0] = 6u8;
        {
            let reader = test_struct_list.as_reader();
            assert_eq!(reader.get_struct_list().unwrap()[0].get_uint8_list().unwrap()[0], 5u8);
        }
    }

    #[test]
    fn test_blob () {
        use test_capnp::TestBlob;

        let mut message = MallocMessageBuilder::new_default();
        let test_blob = message.init_root::<TestBlob::Builder>();

        assert_eq!(test_blob.has_text_field(), false);
        test_blob.set_text_field("abcdefghi");
        assert_eq!(test_blob.has_text_field(), true);

        assert_eq!(test_blob.has_data_field(), false);
        test_blob.set_data_field([0u8, 1u8, 2u8, 3u8, 4u8]);
        assert_eq!(test_blob.has_data_field(), true);

        let test_blob_reader = test_blob.as_reader();

        assert_eq!(test_blob_reader.has_text_field(), true);
        assert_eq!(test_blob_reader.has_data_field(), true);

        assert_eq!(test_blob_reader.get_text_field().unwrap(), "abcdefghi");
        assert!(test_blob_reader.get_data_field().unwrap() == [0u8, 1u8, 2u8, 3u8, 4u8]);

        let text_builder = test_blob.init_text_field(10);
        assert_eq!(test_blob.as_reader().get_text_field().unwrap(),
                   "\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
        let mut writer = std::io::BufWriter::new(text_builder.as_mut_bytes());
        writer.write("aabbccddee".as_bytes()).unwrap();

        let data_builder = test_blob.init_data_field(7);
        assert!(test_blob.as_reader().get_data_field().unwrap() ==
                [0u8,0u8,0u8,0u8,0u8,0u8,0u8]);
        for c in data_builder.mut_iter() {
            *c = 5;
        }
        data_builder[0] = 4u8;

        assert_eq!(test_blob.as_reader().get_text_field().unwrap(), "aabbccddee");
        assert!(test_blob.as_reader().get_data_field().unwrap() == [4u8,5u8,5u8,5u8,5u8,5u8,5u8]);

        let bytes = test_blob.get_text_field().unwrap().as_mut_bytes();
        bytes[4] = 'z' as u8;
        bytes[5] = 'z' as u8;
        assert_eq!(test_blob.as_reader().get_text_field().unwrap(), "aabbzzddee");

        test_blob.get_data_field().unwrap()[2] = 10;
        assert!(test_blob.as_reader().get_data_field().unwrap() == [4u8,5u8,10u8,5u8,5u8,5u8,5u8]);
    }


    #[test]
    fn test_big_struct() {
        use test_capnp::TestBigStruct;

        // Make the first segment small to force allocation of a second segment.
        let mut message = MallocMessageBuilder::new(5, SUGGESTED_ALLOCATION_STRATEGY);

        let bigStruct = message.init_root::<TestBigStruct::Builder>();

        bigStruct.set_bool_field(false);
        bigStruct.set_int8_field(-128);
        bigStruct.set_int16_field(0);
        bigStruct.set_int32_field(1009);

        assert_eq!(bigStruct.has_struct_field(), false);
        let inner = bigStruct.init_struct_field();
        assert_eq!(bigStruct.has_struct_field(), true);
        inner.set_float64_field(0.1234567);

        inner.set_bool_field_b(true);

        bigStruct.set_bool_field(true);


        let bigStructReader = bigStruct.as_reader();
        assert_eq!(bigStructReader.has_struct_field(), true);
        assert_eq!(bigStructReader.get_int8_field(), -128);
        assert_eq!(bigStructReader.get_int32_field(), 1009);

        let innerReader = bigStructReader.get_struct_field().unwrap();
        assert!(!innerReader.get_bool_field_a());
        assert!(innerReader.get_bool_field_b());
        assert_eq!(innerReader.get_float64_field(), 0.1234567);
    }

    #[test]
    fn test_complex_list () {
        use test_capnp::{TestComplexList, AnEnum};

        let mut message = MallocMessageBuilder::new_default();

        let test_complex_list = message.init_root::<TestComplexList::Builder>();

        let enumList = test_complex_list.init_enum_list(100);

        for i in range::<uint>(0, 10) {
            enumList.set(i, AnEnum::Qux);
        }
        for i in range::<uint>(10, 20) {
            enumList.set(i, AnEnum::Bar);
        }

        let text_list = test_complex_list.init_text_list(2);
        text_list.set(0, "garply");
        text_list.set(1, "foo");

        let data_list = test_complex_list.init_data_list(2);
        data_list.set(0, [0u8, 1u8, 2u8]);
        data_list.set(1, [255u8, 254u8, 253u8]);

        let prim_list_list = test_complex_list.init_prim_list_list(2);
        let prim_list = prim_list_list.init(0, 3);
        prim_list.set(0, 5);
        prim_list.set(1, 6);
        prim_list.set(2, 7);
        assert_eq!(prim_list.size(), 3);
        let prim_list = prim_list_list.init(1, 1);
        prim_list.set(0,-1);

        let prim_list_list_list = test_complex_list.init_prim_list_list_list(2);
        let prim_list_list = prim_list_list_list.init(0, 2);
        let prim_list = prim_list_list.init(0, 2);
        prim_list.set(0, 0);
        prim_list.set(1, 1);
        let prim_list = prim_list_list.init(1, 1);
        prim_list.set(0, 255);
        let prim_list_list = prim_list_list_list.init(1, 1);
        let prim_list = prim_list_list.init(0, 3);
        prim_list.set(0, 10);
        prim_list.set(1, 9);
        prim_list.set(2, 8);

        let enum_list_list = test_complex_list.init_enum_list_list(2);
        let enum_list = enum_list_list.init(0, 1);
        enum_list.set(0, AnEnum::Bar);
        let enum_list = enum_list_list.init(1, 2);
        enum_list.set(0, AnEnum::Foo);
        enum_list.set(1, AnEnum::Qux);

        let text_list_list = test_complex_list.init_text_list_list(1);
        text_list_list.init(0,1).set(0, "abc");

        let data_list_list = test_complex_list.init_data_list_list(1);
        data_list_list.init(0,1).set(0, [255, 254, 253]);

        let struct_list_list = test_complex_list.init_struct_list_list(1);
        struct_list_list.init(0,1)[0].set_int8_field(-1);


        let complex_list_reader = test_complex_list.as_reader();
        let enumListReader = complex_list_reader.get_enum_list().unwrap();
        for i in range::<uint>(0,10) {
            assert!(enumListReader[i] == Some(AnEnum::Qux));
        }
        for i in range::<uint>(10,20) {
            assert!(enumListReader[i] == Some(AnEnum::Bar));
        }

        let text_list = complex_list_reader.get_text_list().unwrap();
        assert_eq!(text_list.size(), 2);
        assert_eq!(text_list[0].unwrap(), "garply");
        assert_eq!(text_list[1].unwrap(), "foo");

        let data_list = complex_list_reader.get_data_list().unwrap();
        assert_eq!(data_list.size(), 2);
        assert!(data_list[0].unwrap() == [0u8, 1u8, 2u8]);
        assert!(data_list[1].unwrap() == [255u8, 254u8, 253u8]);

        let prim_list_list = complex_list_reader.get_prim_list_list().unwrap();
        assert_eq!(prim_list_list.size(), 2);
        assert_eq!(prim_list_list[0].unwrap().size(), 3);
        assert!(prim_list_list[0].unwrap()[0] == 5);
        assert!(prim_list_list[0].unwrap()[1] == 6);
        assert!(prim_list_list[0].unwrap()[2] == 7);
        assert!(prim_list_list[1].unwrap()[0] == -1);

        let prim_list_list_list = complex_list_reader.get_prim_list_list_list().unwrap();
        assert!(prim_list_list_list[0].unwrap()[0].unwrap()[0] == 0);
        assert!(prim_list_list_list[0].unwrap()[0].unwrap()[1] == 1);
        assert!(prim_list_list_list[0].unwrap()[1].unwrap()[0] == 255);
        assert!(prim_list_list_list[1].unwrap()[0].unwrap()[0] == 10);
        assert!(prim_list_list_list[1].unwrap()[0].unwrap()[1] == 9);
        assert!(prim_list_list_list[1].unwrap()[0].unwrap()[2] == 8);

        let enum_list_list = complex_list_reader.get_enum_list_list().unwrap();
        assert!(enum_list_list[0].unwrap()[0] == Some(AnEnum::Bar));
        assert!(enum_list_list[1].unwrap()[0] == Some(AnEnum::Foo));
        assert!(enum_list_list[1].unwrap()[1] == Some(AnEnum::Qux));

        assert!(complex_list_reader.get_text_list_list().unwrap()[0].unwrap()[0].unwrap() == "abc");
        assert!(complex_list_reader.get_data_list_list().unwrap()[0].unwrap()[0].unwrap() == [255, 254, 253]);

        assert!(complex_list_reader.get_struct_list_list().unwrap()[0].unwrap()[0].get_int8_field() == -1);
    }

    #[test]
    fn test_defaults() {
        use test_capnp::TestDefaults;

        let mut message = MallocMessageBuilder::new_default();
        let test_defaults = message.init_root::<TestDefaults::Builder>();

        assert_eq!(test_defaults.get_void_field(), ());
        assert_eq!(test_defaults.get_bool_field(), true);
        assert_eq!(test_defaults.get_int8_field(), -123);
        assert_eq!(test_defaults.get_int16_field(), -12345);
        assert_eq!(test_defaults.get_int32_field(), -12345678);
        assert_eq!(test_defaults.get_int64_field(), -123456789012345);
        assert_eq!(test_defaults.get_uint8_field(), 234u8);
        assert_eq!(test_defaults.get_uint16_field(), 45678u16);
        assert_eq!(test_defaults.get_uint32_field(), 3456789012u32);
        assert_eq!(test_defaults.get_uint64_field(), 12345678901234567890u64);
        assert_eq!(test_defaults.get_float32_field(), 1234.5);
        assert_eq!(test_defaults.get_float64_field(), -123e45);

        test_defaults.set_bool_field(false);
        assert_eq!(test_defaults.get_bool_field(), false);
        test_defaults.set_int8_field(63);
        assert_eq!(test_defaults.get_int8_field(), 63);
    }

    #[test]
    fn test_any_pointer() {
        use test_capnp::{TestAnyPointer, TestEmptyStruct};

        let mut message = MallocMessageBuilder::new_default();
        let test_any_pointer = message.init_root::<TestAnyPointer::Builder>();

        let any_pointer = test_any_pointer.init_any_pointer_field();
        any_pointer.set_as_text("xyzzy");

        {
            let reader = test_any_pointer.as_reader();
            assert_eq!(reader.get_any_pointer_field().get_as_text().unwrap(), "xyzzy");
        }

        any_pointer.init_as_struct::<TestEmptyStruct::Builder>();
        any_pointer.get_as_struct::<TestEmptyStruct::Builder>().unwrap();

        {
            let reader = test_any_pointer.as_reader();
            reader.get_any_pointer_field().get_as_struct::<TestEmptyStruct::Reader>().unwrap();
        }

    }

    #[test]
    fn test_writable_struct_pointer() {
        use test_capnp::TestBigStruct;

        let mut message = MallocMessageBuilder::new_default();
        let big_struct = message.init_root::<TestBigStruct::Builder>();

        let struct_field = big_struct.init_struct_field();
        assert_eq!(struct_field.get_uint64_field(), 0);

        struct_field.set_uint64_field(-7);
        assert_eq!(struct_field.get_uint64_field(), -7);
        assert_eq!(big_struct.get_struct_field().unwrap().get_uint64_field(), -7);
        let struct_field = big_struct.init_struct_field();
        assert_eq!(struct_field.get_uint64_field(), 0);
        assert_eq!(struct_field.get_uint32_field(), 0);

        // getting before init is the same as init
        let other_struct_field = big_struct.get_another_struct_field().unwrap();
        assert_eq!(other_struct_field.get_uint64_field(), 0);
        other_struct_field.set_uint32_field(-31);

        let reader = other_struct_field.as_reader();
        big_struct.set_struct_field(reader);
        assert_eq!(big_struct.get_struct_field().unwrap().get_uint32_field(), -31);
        assert_eq!(other_struct_field.get_uint32_field(), -31);
        other_struct_field.set_uint32_field(42);
        assert_eq!(big_struct.get_struct_field().unwrap().get_uint32_field(), -31);
        assert_eq!(other_struct_field.get_uint32_field(), 42);
        assert_eq!(big_struct.get_another_struct_field().unwrap().get_uint32_field(), 42);
    }

    #[test]
    fn test_union() {
        use test_capnp::TestUnion;

        let mut message = MallocMessageBuilder::new_default();
        let union_struct = message.init_root::<TestUnion::Builder>();

        union_struct.get_union0().set_u0f0s0(());
        match union_struct.get_union0().which() {
            Ok(TestUnion::Union0::U0f0s0(())) => {}
            _ => fail!()
        }
        union_struct.init_union0().set_u0f0s1(true);
        match union_struct.get_union0().which() {
            Ok(TestUnion::Union0::U0f0s1(true)) => {}
            _ => fail!()
        }
        union_struct.init_union0().set_u0f0s8(127);
        match union_struct.get_union0().which() {
            Ok(TestUnion::Union0::U0f0s8(127)) => {}
            _ => fail!()
        }

        assert_eq!(union_struct.get_union0().has_u0f0sp(), false);
        union_struct.init_union0().set_u0f0sp("abcdef");
        assert_eq!(union_struct.get_union0().has_u0f0sp(), true);
    }

    #[test]
    fn test_constants() {
        use test_capnp::TestConstants;
        assert_eq!(TestConstants::VOID_CONST, ());
        assert_eq!(TestConstants::BOOL_CONST, true);
        assert_eq!(TestConstants::INT8_CONST, -123);
        assert_eq!(TestConstants::INT16_CONST, -12345);
        assert_eq!(TestConstants::INT32_CONST, -12345678);
        assert_eq!(TestConstants::INT64_CONST, -123456789012345);
        assert_eq!(TestConstants::UINT8_CONST, 234);
        assert_eq!(TestConstants::UINT16_CONST, 45678);
        assert_eq!(TestConstants::UINT32_CONST, 3456789012);
        assert_eq!(TestConstants::UINT64_CONST, 12345678901234567890);
        assert_eq!(TestConstants::FLOAT32_CONST, 1234.5);
        assert_eq!(TestConstants::FLOAT64_CONST, -123e45);
    }

    #[test]
    fn test_set_root() {
        use test_capnp::TestBigStruct;

        let mut message1 = MallocMessageBuilder::new_default();
        let mut message2 = MallocMessageBuilder::new_default();
        let struct1 = message1.init_root::<TestBigStruct::Builder>();
        struct1.set_uint8_field(3);
        message2.set_root(&struct1.as_reader());
        let struct2 = message2.get_root::<TestBigStruct::Builder>().unwrap();

        assert_eq!(struct2.get_uint8_field(), 3u8);
    }

}
