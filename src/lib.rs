// offsetof()-like operation. Will become obsolete when-and-if offsetof() is
// implemented in the core language.
pub macro_rules! field_offset {
    ($container:ty : $field:ident) => (unsafe {
        let nullptr = 0 as * const $container;
        let fieldptr: * const _ = &((*nullptr).$field);
        fieldptr as usize
    })
}

#[cfg(test)]
pub mod test {
    pub struct MyStruct {
        field1: i32,
        field2: i32,
        field3: i32,
    }

    impl MyStruct {
        pub fn field1_offset() -> usize {
            field_offset!(MyStruct:field1)
        }
        pub fn field2_offset() -> usize {
            field_offset!(MyStruct:field2)
        }
        pub fn field3_offset() -> usize {
            field_offset!(MyStruct:field3)
        }
    }

    #[test]
    fn test_field_offset() {
        let offset_field1_meth1 = unsafe {
            let nullptr = 0 as * const MyStruct;
            let fieldptr: * const _ = &((*nullptr).field1);
            fieldptr as usize
        };
        let offset_field1_meth2 = MyStruct::field1_offset();
        assert_eq!(offset_field1_meth1, offset_field1_meth2);
        assert_eq!(0, offset_field1_meth2);
        assert_eq!(4, MyStruct::field2_offset());
        assert_eq!(8, MyStruct::field3_offset());
    }
}
