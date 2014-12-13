#![feature(macro_rules)]

// offsetof()-like operation. Will become obsolete when-and-if offsetof() is
// implemented in the core language.
pub macro_rules! field_offset(
    ($container:ty . $field:ident : $fieldtype:ty) => (unsafe {
        let nullptr = 0 as * const $container;
        let fieldptr: * const $fieldtype = &((*nullptr).$field);
        fieldptr.to_uint()
    })
)
// container_of()-like operation.
pub macro_rules! from_field(
    ($name:ident($container:ty . $field:ident : $fieldtype:ty)) =>
    (fn $name(arg: &$fieldtype) -> &$container {
        unsafe {
            let argp: * const $fieldtype = &*arg;
            let argi = argp.to_uint();
            let containeri =
                argi - field_offset!($container.$field:$fieldtype);
            let containerp: * const $container =
                containeri as * const $container;
            &*containerp
        }
    })
)
pub macro_rules! to_field(
    ($name:ident($container:ty . $field:ident : $fieldtype:ty)) =>
    (fn $name(arg: &$container) -> &$fieldtype {
        &(*arg).$field
    })
)

#[cfg(test)]
pub mod test {
    pub struct MyStruct {
        field1: i32,
        field2: i32,
        field3: i32,
    }

    impl MyStruct {
        pub fn field1_offset() -> uint {
            field_offset!(MyStruct.field1:i32)
        }
        pub fn field2_offset() -> uint {
            field_offset!(MyStruct.field2:i32)
        }
        pub fn field3_offset() -> uint {
            field_offset!(MyStruct.field3:i32)
        }
        fn from_field1_meth1(field1: &i32) -> &MyStruct {
            unsafe {
                let field1p: * const i32 = &*field1;
                let field1i = field1p.to_uint();
                let selfptr = field1i - field_offset!(MyStruct.field1:i32);
                let typedselfptr: * const MyStruct =
                    selfptr as * const MyStruct;
                &(*typedselfptr)
            }
        }
        from_field!(from_field1_meth2(MyStruct.field1:i32))
        from_field!(from_field2(MyStruct.field2:i32))
        from_field!(from_field3(MyStruct.field3:i32))
        to_field!(to_field1(MyStruct.field1:i32))    
        to_field!(to_field2(MyStruct.field2:i32))    
        to_field!(to_field3(MyStruct.field3:i32))    
    }

    #[test]
    fn test_field_offset() {
        let offset_field1_meth1 = unsafe {
            let nullptr = 0 as * const MyStruct;
            let fieldptr: * const i32 = &((*nullptr).field1);
            fieldptr.to_uint()
        };
        let offset_field1_meth2 = MyStruct::field1_offset();
        assert_eq!(offset_field1_meth1, offset_field1_meth2);
        assert_eq!(0, offset_field1_meth2);
        assert_eq!(4, MyStruct::field2_offset());
        assert_eq!(8, MyStruct::field3_offset());
    }

    #[test]
    fn test_from_field() {
        let m = MyStruct { field1: 0, field2: 0, field3: 0 };
        let m1: * const MyStruct = &m;
        let m2_meth1: * const MyStruct =
            MyStruct::from_field1_meth1(&m.field1);
        assert_eq!(m2_meth1, m1);
        assert_eq!(
            m1, MyStruct::from_field1_meth2(&m.field1) as * const MyStruct);
        assert_eq!(
            m1, MyStruct::from_field2(&m.field2) as * const MyStruct);
        assert_eq!(
            m1, MyStruct::from_field3(&m.field3) as * const MyStruct);
    }

    #[test]
    fn test_to_field() {
        let m = MyStruct { field1: 0, field2: 0, field3: 0 };
        let mp: * const MyStruct = &m;
        let field2p = MyStruct::to_field2(&m);
        let m2p = MyStruct::from_field2(field2p) as * const MyStruct;
        assert_eq!(mp, m2p);
    }
}
