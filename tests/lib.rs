#![cfg(test)]
#[macro_use]
extern crate containerof;
use containerof::*;

#[derive(Clone)]
struct MyStruct {
    field1: i32,
    field2: i32,
    field3: i32,
}

struct MyStructField2_Meth1(usize);
impl Intrusive for MyStructField2_Meth1 {
    type Container = MyStruct;
    type Field = i32;

    unsafe fn from_container(c: Box<MyStruct>) -> Self {
        let cp: *const MyStruct = ::std::mem::transmute(c);
        MyStructField2_Meth1(::std::mem::transmute(&((*cp).field2)))
    }
    unsafe fn into_container(self) -> Box<MyStruct> {
        let fieldptr = self.0;
        let containerptr = fieldptr - containerof_field_offset!(MyStruct:field2);
        ::std::mem::transmute(containerptr)
    }
    fn as_container<'a>(&'a self) -> &'a MyStruct {
        unsafe {
            let fieldptr = self.0;
            let containerptr = fieldptr - containerof_field_offset!(MyStruct:field2);
            ::std::mem::transmute(containerptr)
        }
    }
    fn as_container_mut<'a>(&'a mut self) -> &'a mut MyStruct {
        unsafe { ::std::mem::transmute(self.as_container()) }
    }
    unsafe fn from_field(c: Box<i32>) -> Self {
        MyStructField2_Meth1(::std::mem::transmute(c))
    }
    unsafe fn into_field(self) -> Box<i32> {
        ::std::mem::transmute(self.0)
    }
    fn as_field(&self) -> &i32 {
        unsafe { ::std::mem::transmute(self.0) }
    }
    fn as_field_mut(&mut self) -> &mut i32 {
        unsafe { ::std::mem::transmute(self.0) }
    }

    unsafe fn from_alias(ia: IntrusiveAlias) -> Self {
        ::std::mem::transmute(ia)
    }
    unsafe fn into_alias(self) -> IntrusiveAlias {
        ::std::mem::transmute(self)
    }
    unsafe fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias {
        ::std::mem::transmute(self)
    }
    unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut IntrusiveAlias {
        ::std::mem::transmute(self)
    }
    unsafe fn of_alias<'a>(ia: &'a IntrusiveAlias) -> &'a Self {
        ::std::mem::transmute(ia)
    }
    unsafe fn of_alias_mut<'a>(ia: &'a mut IntrusiveAlias) -> &'a mut Self {
        ::std::mem::transmute(ia)
    }
}
containerof_intrusive!(MyStructField2_Meth2 = MyStruct:field2::i32);

#[test]
fn test_field_offset() {
    let ms = MyStruct { field1: 1, field2: 2, field3: 3 };
    let ms_addr:usize = unsafe { ::std::mem::transmute(&ms) };
    let ms_field1_addr:usize = unsafe { ::std::mem::transmute(&ms.field1) };
    let ms_field2_addr:usize = unsafe { ::std::mem::transmute(&ms.field2) };
    let ms_field3_addr:usize = unsafe { ::std::mem::transmute(&ms.field3) };
    assert_eq!(ms_field1_addr - ms_addr, containerof_field_offset!(MyStruct:field1));
    assert_eq!(ms_field2_addr - ms_addr, containerof_field_offset!(MyStruct:field2));
    assert_eq!(ms_field3_addr - ms_addr, containerof_field_offset!(MyStruct:field3));
}
#[test]
fn test_intrusive_container_roundtrip() {
    let mc1 = Box::new(MyStruct { field1: 1, field2: 2, field3: 3 });
    let mc1_addr: usize = unsafe { ::std::mem::transmute(&*mc1) };

    let mcfield: MyStructField2_Meth1 = unsafe { Intrusive::from_container(mc1) };
    let mcfieldcontainer: usize = unsafe {
        ::std::mem::transmute(mcfield.as_container())
    };

    assert_eq!(mc1_addr, mcfieldcontainer);
    ::std::mem::drop(mcfieldcontainer);

    let mc2 = unsafe { mcfield.into_container() };
    let mc2_addr: usize = unsafe { ::std::mem::transmute_copy(&mc2) };
    assert_eq!(mc1_addr, mc2_addr);
}

#[test]
fn test_intrusive_field_roundtrip() {
    let mc1 = Box::new(MyStruct { field1: 1, field2: 2, field3: 3 });
    let mc1_addr: usize = unsafe { ::std::mem::transmute(&*mc1) };
    let mc1_field_addr: usize = unsafe { ::std::mem::transmute(&mc1.field2) };

    let mcfield: MyStructField2_Meth1 = unsafe { Intrusive::from_container(mc1) };
    let mcfield_as_addr: usize = unsafe {
        ::std::mem::transmute(mcfield.as_field())
    };
    assert_eq!(mc1_field_addr, mcfield_as_addr);

    let mcfield = unsafe { mcfield.into_field() };
    let mcfield_addr = unsafe { ::std::mem::transmute(&*mcfield) };
    assert_eq!(mc1_field_addr, mcfield_addr);

    let mcfield: MyStructField2_Meth1 = unsafe { Intrusive::from_field(mcfield) };
    let mc2 = unsafe { mcfield.into_container() };
    let mc2_addr: usize = unsafe { ::std::mem::transmute(&*mc2) };
    assert_eq!(mc1_addr, mc2_addr);
}
