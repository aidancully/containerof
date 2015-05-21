//! Intrusive structure support in Rust.
//!
//! An intrusive structure is a general-purpose structure directly
//! embedded within a containing structure, in order to add that
//! general-purpose facility to the container. As an example, one
//! might use an intrusive "link" structure to allow objects to be
//! organized in a linked-list:
//!
//! # Example
//!
//! ```test_harness
//! # #[macro_use]
//! # extern crate containerof;
//! # use containerof::*;
//! struct Link {
//!     next: Option<ContainerLink>,
//! }
//! struct List {
//!     head: Option<ContainerLink>,
//!     tail: Option<ContainerLink>,
//! }
//! 
//! struct Container {
//!     link: Link,
//! }
//! containerof_intrusive!(ContainerLink = Container:link::Link);
//! ```

use std::ops;
use std::mem;
use std::marker;

/// Implement C-like `offsetof` macro in Rust. Will become obsolete
/// when-and-if offsetof() is implemented in the core language.
#[macro_export]
macro_rules! containerof_field_offset {
    ($container:ty : $field:ident) => (unsafe {
        &(*(0usize as *const $container)).$field as *const _ as usize
    })
}

/// Define a type representing the translation between an intrusive
/// field and its containing structure.
#[macro_export]
macro_rules! containerof_intrusive {
    ($nt:ident = $container:ty : $field:ident :: $fieldtype:ty) => (
        containerof_intrusive!(_decl $nt);
        containerof_intrusive!(_impl $nt = $container : $field :: $fieldtype);
        );
    (pub $nt:ident = $container:ty : $field:ident :: $fieldtype:ty) => (
        containerof_intrusive!(_decl pub $nt);
        containerof_intrusive!(_impl $nt = $container : $field :: $fieldtype);
        );
    // below are implementation details. you should not invoke these
    // macro variants directly.
    (_decl $nt:ident) => (
        struct $nt($crate::IntrusiveAlias);
        );
    (_decl pub $nt:ident) => (
        pub struct $nt($crate::IntrusiveAlias);
        );
    (_impl $nt:ident = $container:ty : $field:ident :: $fieldtype:ty) => (
        impl $crate::IntrusiveBase for $nt {
            type Container = $container;
            type Field = $fieldtype;
            #[inline]
            fn offset() -> usize {
                containerof_field_offset!($container : $field)
            }
            #[inline]
            unsafe fn new(ia: $crate::IntrusiveAlias) -> $nt {
                $nt(ia)
            }
            #[inline]
            unsafe fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias {
                ::std::mem::transmute(self as *const _)
            }
        }
        );
}

/// Alias that has the same representation as an intrusive type. The
/// idea is to be able to use this alias for intrusive facility
/// implementations, by defining the "true" implementation of the
/// facility to use the single (but type-unsafe) IntrusiveAlias type,
/// while allowing type-safe wrapper implementations to delegate their
/// behavior to the implementation function.
#[derive(PartialEq,Eq,Copy,Clone,Debug)]
pub struct IntrusiveAlias(pub usize);
impl IntrusiveAlias {
    pub fn new(addr: usize) -> IntrusiveAlias {
        IntrusiveAlias(addr)
    }
    pub unsafe fn new_of<T>(addr: &T) -> IntrusiveAlias {
        IntrusiveAlias::new(addr as *const _ as usize)
    }
    pub fn get_address(&self) -> usize {
        self.0
    }
}

// FIXME: this wants to be a linear type, but that requires linear-type
// support in the language.
pub struct OwnBox<T> {
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<T>,
}
impl<T> OwnBox<T> {
    pub fn get_address(&self) -> usize {
        self.pointer.0
    }
    pub unsafe fn from_alias(pointer: IntrusiveAlias) -> OwnBox<T> {
        OwnBox { pointer: pointer, marker: marker::PhantomData }
    }
    pub unsafe fn from_ref(p: &mut T) -> OwnBox<T> {
        OwnBox::from_alias(IntrusiveAlias::new_of(p))
    }
    pub unsafe fn into_alias(self) -> IntrusiveAlias {
        let rval = self.pointer;
        mem::forget(self);
        rval
    }
    pub fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias {
        &self.pointer
    }
    pub unsafe fn from_box(b: Box<T>) -> OwnBox<T> {
        OwnBox { pointer: IntrusiveAlias::new(mem::transmute(b)), marker: marker::PhantomData }
    }
    pub unsafe fn into_box(self) -> Box<T> {
        mem::transmute(self.into_alias().get_address())
    }
}
impl<T> ops::Deref for OwnBox<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { mem::transmute(self.get_address()) }
    }
}
impl<T> ops::DerefMut for OwnBox<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { mem::transmute(self.get_address()) }
    }
}
impl<T> ops::Drop for OwnBox<T> {
    fn drop(&mut self) {
        // should have been consumed via "into_box" or "into_alias".
        // TODO: want a better way to encourage linearity on this type.
        //panic!("containerof::OwnBox should be treated as linear!");
    }
}

#[derive(Debug)]
pub struct BorrowBox<'a, T: 'a> {
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<&'a T>,
}
impl<'a, T> BorrowBox<'a, T> where T: Intrusive {
    pub fn new(source: &'a T) -> BorrowBox<'a, T> {
        unsafe { BorrowBox::new_from(IntrusiveAlias::new_of(source), source) }
    }
    pub unsafe fn new_from<U>(pointer: IntrusiveAlias, _lifetime: &'a U) -> BorrowBox<'a, T> {
        BorrowBox {
            pointer: pointer,
            marker: marker::PhantomData,
        }
    }
}
impl<'a, T> ops::Deref for BorrowBox<'a, T> where T: Intrusive {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        unsafe { Intrusive::of_alias(&self.pointer) }
    }
}

#[derive(Debug)]
pub struct BorrowBoxMut<'a, T: 'a> where T: Intrusive {
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<&'a mut T>,
}
impl<'a, T> BorrowBoxMut<'a, T> where T: Intrusive {
    pub fn new(source: &'a mut T) -> BorrowBoxMut<'a, T> {
        unsafe { BorrowBoxMut::new_from(IntrusiveAlias::new_of(source), source) }
    }
    pub unsafe fn new_from<U>(pointer: IntrusiveAlias, _lifetime: &'a mut U) -> BorrowBoxMut<'a, T> {
        BorrowBoxMut {
            pointer: pointer,
            marker: marker::PhantomData,
        }
    }
}
impl<'a, T> ops::Deref for BorrowBoxMut<'a, T> where T: Intrusive {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        unsafe { Intrusive::of_alias(&self.pointer) }
    }
}
impl<'a, T> ops::DerefMut for BorrowBoxMut<'a, T> where T: Intrusive {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        unsafe { Intrusive::of_alias_mut(&mut self.pointer) }
    }
}

pub trait IntrusiveBase {
    /// Type of containing structure.
    type Container;
    /// Type of intrusive field within containing structure.
    type Field;
    // TODO: would also like to see an "offset" associated const, but
    // Rust doesn't seem to support these yet.
    /// Returns offset of intrusive field within containing structure.
    fn offset() -> usize;

    /// Ownership-moving translation from generic intrusive pointer
    /// alias to type-safe intrusive pointer.
    unsafe fn new(IntrusiveAlias) -> Self;

    /// Allow using type-safe intrusive pointer as generic intrusive pointer.
    unsafe fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias;
}

/// Trait defining routines for translation between containing
/// structure and intrusive field. The only implementors of this type
/// should be the pointer-types defined by the `containerof_intrusive`
/// macro.
pub trait Intrusive: IntrusiveBase {
    /// Ownership-moving translation from generic intrusive pointer
    /// alias to type-safe intrusive pointer.
    unsafe fn from_alias(IntrusiveAlias) -> Self;

    /// Ownership-moving translation from type-safe intrusive pointer
    /// to generic intrusive pointer.
    unsafe fn into_alias(self) -> IntrusiveAlias;

    /// Allow using type-safe intrusive pointer as mutable generic
    /// intrusive pointer.
    unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut IntrusiveAlias;

    /// Allow using generic intrusive pointer as type-safe intrusive
    /// pointer.
    unsafe fn of_alias<'a>(&'a IntrusiveAlias) -> &'a Self;

    /// Allow using generic intrusive pointer as mutable type-safe
    /// intrusive pointer.
    unsafe fn of_alias_mut<'a>(ia: &'a mut IntrusiveAlias) -> &'a mut Self;

    /// Represent ownership of a container as ownership of an Intrusive
    /// pointer type. (Inverse of `into_container`.)
    fn from_container(OwnBox<Self::Container>) -> Self;

    /// Represent ownership of an Intrusive pointer type as ownership of
    /// its container. (Inverse of `from_container`.)
    fn into_container(self) -> OwnBox<Self::Container>;

    fn of_container<'a>(&'a Self::Container) -> BorrowBox<'a, Self>;
    fn of_container_mut<'a>(&'a mut Self::Container) -> BorrowBoxMut<'a, Self>;

    /// Grant referential access to the container of this intrusive
    /// pointer type.
    fn as_container<'a>(&'a self) -> &'a Self::Container;
    /// Grant mutable referential access to the container of this
    /// intrusive pointer type.
    fn as_container_mut<'a>(&'a mut self) -> &'a mut Self::Container;

    /// Assuming the "field" is a field in the container object, take
    /// ownership of the field as an intrusive pointer, allowing
    /// eventual translation back to the container. (Inverse of
    /// `into_field`.)
    unsafe fn from_field(OwnBox<Self::Field>) -> Self;
    
    /// Represent ownership of the container object as ownership of
    /// the intrusive field in the object. (Inverse of `from_field`.)
    unsafe fn into_field(self) -> OwnBox<Self::Field>;

    unsafe fn of_field<'a>(&'a Self::Field) -> BorrowBox<'a, Self>;
    unsafe fn of_field_mut<'a>(&'a mut Self::Field) -> BorrowBoxMut<'a, Self>;

    /// Grant referential access to the intrusive field represented by
    /// this intrusive pointer.
    fn as_field<'a>(&'a self) -> &'a Self::Field;

    /// Grant mutable referential access to the intrusive field represented by
    /// this intrusive pointer.
    fn as_field_mut<'a>(&'a mut self) -> &'a mut Self::Field;
}

impl<T: IntrusiveBase> Intrusive for T {
    #[inline]
    unsafe fn from_alias(ia: IntrusiveAlias) -> T {
        <T as IntrusiveBase>::new(ia)
    }
    #[inline]
    unsafe fn into_alias(self) -> IntrusiveAlias {
        *self.as_alias()
    }
    #[allow(mutable_transmutes)]
    #[inline]
    unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut IntrusiveAlias {
        mem::transmute(self.as_alias())
    }
    #[inline]
    unsafe fn of_alias<'a>(ia: &'a IntrusiveAlias) -> &'a T {
        mem::transmute(ia as *const _)
    }
    #[inline]
    unsafe fn of_alias_mut<'a>(ia: &'a mut IntrusiveAlias) -> &'a mut T {
        mem::transmute(ia as *const _)
    }
    #[inline]
    fn from_container(c: OwnBox<T::Container>) -> Self {
        unsafe {
            let addr = c.as_alias().get_address() + <T as IntrusiveBase>::offset();
            mem::forget(c);
            <T as Intrusive>::from_alias(IntrusiveAlias(addr))
        }
    }
    #[inline]
    fn into_container(self) -> OwnBox<T::Container> {
        let fieldptr = unsafe { self.as_alias().get_address() };
        let containerptr = fieldptr - <T as IntrusiveBase>::offset();
        unsafe { OwnBox::from_alias(IntrusiveAlias::new(containerptr)) }
    }
    #[inline]
    fn of_container<'a>(container: &'a T::Container) -> BorrowBox<'a, T> {
        let addr = container as *const _ as usize;
        let fieldptr = addr + <T as IntrusiveBase>::offset();
        unsafe { BorrowBox::new_from(IntrusiveAlias::new(fieldptr), container) }
    }
    #[inline]
    fn of_container_mut<'a>(container: &'a mut T::Container) -> BorrowBoxMut<'a, T> {
        let addr = container as *mut _ as usize;
        let fieldptr = addr + <T as IntrusiveBase>::offset();
        unsafe { BorrowBoxMut::new_from(IntrusiveAlias::new(fieldptr), container) }
    }
    #[inline]
    fn as_container<'a>(&'a self) -> &'a T::Container {
        unsafe {
            let fieldptr = self.as_alias().get_address();
            let containerptr = fieldptr - <T as IntrusiveBase>::offset();
            &*(containerptr as *const T::Container)
        }
    }
    #[allow(mutable_transmutes)]
    #[inline]
    fn as_container_mut<'a>(&'a mut self) -> &'a mut T::Container {
        unsafe { mem::transmute(self.as_container()) }
    }
    #[inline]
    unsafe fn from_field(c: OwnBox<T::Field>) -> T {
        let addr = c.as_alias().get_address();
        mem::forget(c);
        <T as Intrusive>::from_alias(IntrusiveAlias::new(addr))
    }
    #[inline]
    unsafe fn into_field(self) -> OwnBox<T::Field> {
        OwnBox::from_alias(IntrusiveAlias(self.as_alias().get_address()))
    }
    #[inline]
    unsafe fn of_field<'a>(field: &'a T::Field) -> BorrowBox<'a, T> {
        BorrowBox::new_from(IntrusiveAlias::new_of(field), field)
    }
    #[inline]
    unsafe fn of_field_mut<'a>(field: &'a mut T::Field) -> BorrowBoxMut<'a, T> {
        BorrowBoxMut::new_from(IntrusiveAlias::new_of(field), field)
    }
    #[inline]
    fn as_field<'a>(&'a self) -> &'a T::Field {
        unsafe { &*(self.as_alias().get_address() as *const _) }
    }
    #[inline]
    fn as_field_mut<'a>(&'a mut self) -> &'a mut T::Field {
        unsafe { &mut *(self.as_alias().get_address() as *mut _) }
    }
}
