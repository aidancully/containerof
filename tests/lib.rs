#![cfg(test)]
#[macro_use(field_offset, intrusive)]
extern crate intrusive;
use intrusive::*;

#[derive(Clone)]
struct MyStruct {
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

struct MyStructField2_Meth1(usize);
impl ::intrusive::Intrusive for MyStructField2_Meth1 {
    type Container = MyStruct;
    type Field = i32;

    fn move_from(c: Box<MyStruct>) -> Self {
        unsafe {
            let cp: *const MyStruct = ::std::mem::transmute(c);
            MyStructField2_Meth1(::std::mem::transmute(&((*cp).field2)))
        }
    }
    fn container_of(self) -> Box<MyStruct> {
        unsafe {
            let fieldptr = self.0;
            let containerptr = fieldptr - field_offset!(MyStruct:field2);
            ::std::mem::transmute(containerptr)
        }
    }
    fn as_field(&self) -> &i32 {
        unsafe {
            ::std::mem::transmute(self.0)
        }
    }
    fn as_field_mut(&mut self) -> &mut i32 {
        unsafe {
            ::std::mem::transmute(self.0)
        }
    }
}
intrusive!(MyStructField2_Meth2 = MyStruct:field2::i32);

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
#[test]
fn test_intrusive_meth1() {
    let mc1 = Box::new(MyStruct { field1: 1, field2: 2, field3: 3 });
    let mc1_addr: usize = unsafe { ::std::mem::transmute_copy(&mc1) };
    let mcfield: MyStructField2_Meth1 = ::intrusive::Intrusive::move_from(mc1);
    let mc2 = mcfield.container_of();
    let mc2_addr: usize = unsafe { ::std::mem::transmute_copy(&mc2) };
    assert_eq!(mc1_addr, mc2_addr);
}
#[test]
fn test_intrusive_meth2() {
    let mc1 = Box::new(MyStruct { field1: 1, field2: 2, field3: 3 });
    let mc1_addr: usize = unsafe { ::std::mem::transmute_copy(&mc1) };
    let mut mcfield: MyStructField2_Meth2 = ::intrusive::Intrusive::move_from(mc1);
    *(mcfield.as_field_mut()) = 10;
    let mc2 = mcfield.container_of();
    let mc2_addr: usize = unsafe { ::std::mem::transmute_copy(&mc2) };
    assert_eq!(mc1_addr, mc2_addr);
    assert_eq!(10i32, mc2.field2);
}
