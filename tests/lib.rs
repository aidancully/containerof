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
struct MyStructField2_Meth1(usize);
impl Intrusive for MyStructField2_Meth1 {
    type Container = MyStruct;
    type Field = i32;

    fn from_container(c: OwnBox<MyStruct>) -> Self {
        unsafe {
            let c = c.into_alias().0;
            let cp: *const MyStruct = ::std::mem::transmute(c);
            MyStructField2_Meth1(::std::mem::transmute(&((*cp).field2)))
        }
    }
    fn into_container(self) -> OwnBox<MyStruct> {
        let fieldptr = self.0;
        let containerptr = fieldptr - containerof_field_offset!(MyStruct:field2);
        unsafe { OwnBox::from_alias(IntrusiveAlias(containerptr)) }
    }
    fn of_container<'a>(container: &'a MyStruct) -> BorrowBox<'a, MyStructField2_Meth1> {
        let addr = container as *const _ as usize;
        let fieldptr = addr + containerof_field_offset!(MyStruct:field2);
        unsafe { BorrowBox::new_from(IntrusiveAlias(fieldptr), container) }
    }
    fn of_container_mut<'a>(container: &'a mut MyStruct) -> BorrowBoxMut<'a, MyStructField2_Meth1> {
        let addr = container as *mut _ as usize;
        let fieldptr = addr + containerof_field_offset!(MyStruct:field2);
        unsafe { BorrowBoxMut::new_from(IntrusiveAlias(fieldptr), container) }
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
    unsafe fn from_field(c: OwnBox<i32>) -> Self {
        let addr = c.get_address();
        std::mem::forget(c);
        MyStructField2_Meth1(addr)
    }
    unsafe fn into_field(self) -> OwnBox<i32> {
        OwnBox::from_alias(IntrusiveAlias(self.0))
    }
    unsafe fn of_field<'a>(field: &'a i32) -> BorrowBox<'a, MyStructField2_Meth1> {
        unsafe { BorrowBox::new_from(IntrusiveAlias(field as *const _ as usize), field) }
    }
    unsafe fn of_field_mut<'a>(field: &'a mut i32) -> BorrowBoxMut<'a, MyStructField2_Meth1> {
        unsafe { BorrowBoxMut::new_from(IntrusiveAlias(field as *mut _ as usize), field) }
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
    let mc1 = Box::new(MyStruct { field1: 1, field2: 2, field3: 3 });
    let mc1 = unsafe { OwnBox::from_box(mc1) };
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
        assert_eq!(2, *mcfield.as_target().as_field());
    }
    {
        let mut mcfield = <MyStructField2_Meth1 as Intrusive>::of_container_mut(&mut mc);
        assert_eq!(2, *mcfield.as_target().as_field());
        *mcfield.as_target_mut().as_field_mut() = 10;
        assert_eq!(10, *mcfield.as_target().as_field());
    }
    assert_eq!(10, mc.field2);
}
