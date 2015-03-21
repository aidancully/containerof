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

#[derive(Debug)]
#[allow(non_camel_case_types)]
struct MyStructField2_Meth1(usize);
impl IntrusiveBase for MyStructField2_Meth1 {
    type Container = MyStruct;
    type Field = i32;

    fn offset() -> usize {
        containerof_field_offset!(MyStruct:field2)
    }
    unsafe fn new(ia: IntrusiveAlias) -> Self {
        MyStructField2_Meth1(ia.get_address())
    }
    unsafe fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias {
        ::std::mem::transmute(self as *const _)
    }
}
//containerof_intrusive!(MyStructField2_Meth2 = MyStruct:field2::i32);

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
    let mc1 = unsafe { OwnBox::from_box(mc1) };
    let mc1_addr = mc1.get_address();

    let mcfield: MyStructField2_Meth1 = Intrusive::from_container(mc1);
    let mcfieldcontainer: usize = unsafe {
        ::std::mem::transmute(mcfield.as_container())
    };

    assert_eq!(mc1_addr, mcfieldcontainer);
    ::std::mem::drop(mcfieldcontainer);

    let mc2 = mcfield.into_container();
    let mc2_addr = mc2.get_address();
    assert_eq!(mc1_addr, mc2_addr);

    let _ = unsafe { mc2.into_box() };
}

#[test]
fn test_intrusive_field_roundtrip() {
    let mc1 = unsafe {
        OwnBox::from_box(Box::new(MyStruct { field1: 1, field2: 2, field3: 3 }))
    };
    let mc1_addr = mc1.get_address();
    let mc1_field_addr: usize = unsafe { ::std::mem::transmute(&mc1.field2) };

    let mcfield: MyStructField2_Meth1 = Intrusive::from_container(mc1);
    let mcfield_as_addr = mcfield.as_field() as *const _ as usize;
    assert_eq!(mc1_field_addr, mcfield_as_addr);

    let mcfield = unsafe { mcfield.into_field() };
    let mcfield_addr = unsafe { ::std::mem::transmute(&*mcfield) };
    assert_eq!(mc1_field_addr, mcfield_addr);

    let mcfield: MyStructField2_Meth1 = unsafe { Intrusive::from_field(mcfield) };
    let mc2 = mcfield.into_container();
    let mc2_addr = mc2.get_address();
    assert_eq!(mc1_addr, mc2_addr);

    let _ = unsafe { mc2.into_box() };
}

// FIXME: how can we test that compilations fail with Cargo?
// it's part of the contract of borrow_box() that certain
// orders-of-operation are illegal, I'd like to have explicit
// tests that I can leave in place to enforce that compilation
// fails under those circumstances.
#[test]
fn test_borrow_box() {
    let mut mc = MyStruct { field1: 1, field2: 2, field3: 3 };
    {
        let mcfield = <MyStructField2_Meth1 as Intrusive>::of_container(&mc);
        assert_eq!(2, *mcfield.as_field());
    }
    {
        let mut mcfield = <MyStructField2_Meth1 as Intrusive>::of_container_mut(&mut mc);
        assert_eq!(2, *mcfield.as_field());
        *mcfield.as_field_mut() = 10;
        assert_eq!(10, *mcfield.as_field());
    }
    assert_eq!(10, mc.field2);
}
