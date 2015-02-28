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

#![feature(unsafe_destructor)]
#[crate_id = "containerof"]
use std::ops;
use std::mem;
use std::marker;

/// Implement C-like `offsetof` macro in Rust. Will become obsolete
/// when-and-if offsetof() is implemented in the core language.
#[macro_export]
#[unstable = "Will go away if-and-when Rust implements this function directly."]
macro_rules! containerof_field_offset {
    ($container:ty : $field:ident) => (unsafe {
        &(*(0usize as *const $container)).$field as *const _ as usize
    })
}

/// Define a new type that implements the `Intrusive` trait for a field
/// within a type.
#[macro_export]
#[unstable = "Experimental API."]
macro_rules! containerof_intrusive {
    ($nt:ident = $container:ty : $field:ident :: $fieldtype:ty) => (
        // FIXME: $nt should really be a linear type (it is an error to
        // drop an instance of $nt, as the container needs to be recovered
        // for drop to succeed, so drops should be prevented at
        // compiler-level), but Rust doesn't yet support linear types.
        struct $nt(usize);
        impl $crate::Intrusive for $nt {
            type Container = $container;
            type Field = $fieldtype;

            #[inline]
            fn from_container(c: $crate::OwnBox<$container>) -> Self {
                unsafe {
                    let c = c.into_alias().0;
                    let cp: *const $container = ::std::mem::transmute(c);
                    $nt(::std::mem::transmute(&((*cp).$field)))
                }
            }
            #[inline]
            fn into_container(self) -> $crate::OwnBox<$container> {
                let fieldptr = self.0;
                let containerptr = fieldptr - containerof_field_offset!($container:$field);
                unsafe { $crate::OwnBox::from_alias($crate::IntrusiveAlias(containerptr)) }
            }
            #[inline]
            fn of_container<'a>(container: &'a $container) -> $crate::BorrowBox<'a, $nt> {
                let addr = container as *const _ as usize;
                let fieldptr = addr + containerof_field_offset!($container:$field);
                unsafe { $crate::BorrowBox::new_from($crate::IntrusiveAlias(fieldptr), container) }
            }
            #[inline]
            fn of_container_mut<'a>(container: &'a mut $container) -> $crate::BorrowBoxMut<'a, $nt> {
                let addr = container as *mut _ as usize;
                let fieldptr = addr + containerof_field_offset!($container:$field);
                unsafe { $crate::BorrowBoxMut::new_from($crate::IntrusiveAlias(fieldptr), container) }
            }
            #[inline]
            fn as_container<'a>(&'a self) -> &'a $container {
                unsafe {
                    let fieldptr = self.0;
                    let containerptr = fieldptr - containerof_field_offset!($container:$field);
                    ::std::mem::transmute(containerptr)
                }
            }
            #[inline]
            fn as_container_mut<'a>(&'a mut self) -> &'a mut $container {
                unsafe { ::std::mem::transmute(self.as_container()) }
            }

            #[inline]
            unsafe fn from_field(c: $crate::OwnBox<$fieldtype>) -> Self {
                let addr = c.get_address();
                ::std::mem::forget(c);
                $nt(addr)
            }
            #[inline]
            unsafe fn into_field(self) -> $crate::OwnBox<$fieldtype> {
                $crate::OwnBox::from_alias($crate::IntrusiveAlias(self.0))
            }
            #[inline]
            unsafe fn of_field<'a>(field: &'a $fieldtype) -> $crate::BorrowBox<'a, $nt> {
                unsafe { $crate::BorrowBox::new_from($crate::IntrusiveAlias(field as *const _ as usize), field) }
            }
            #[inline]
            unsafe fn of_field_mut<'a>(field: &'a mut $fieldtype) -> $crate::BorrowBoxMut<'a, $nt> {
                unsafe { $crate::BorrowBoxMut::new_from($crate::IntrusiveAlias(field as *mut _ as usize), field) }
            }
            #[inline]
            fn as_field<'a>(&'a self) -> &'a $fieldtype {
                unsafe { ::std::mem::transmute(self.0) }
            }
            #[inline]
            fn as_field_mut<'a>(&'a mut self) -> &'a mut $fieldtype {
                unsafe { ::std::mem::transmute(self.0) }
            }

            unsafe fn from_alias(ia: $crate::IntrusiveAlias) -> Self {
                ::std::mem::transmute(ia)
            }
            unsafe fn into_alias(self) -> $crate::IntrusiveAlias {
                ::std::mem::transmute(self)
            }
            unsafe fn as_alias<'a>(&'a self) -> &'a $crate::IntrusiveAlias {
                ::std::mem::transmute(self)
            }
            unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut $crate::IntrusiveAlias {
                ::std::mem::transmute(self)
            }
            unsafe fn of_alias<'a>(ia: &'a $crate::IntrusiveAlias) -> &'a Self {
                ::std::mem::transmute(ia)
            }
            unsafe fn of_alias_mut<'a>(ia: &'a mut $crate::IntrusiveAlias) -> &'a mut Self {
                ::std::mem::transmute(ia)
            }
        }
    )
}

/// Alias that has the same representation as an intrusive type. The
/// idea is to be able to use this alias for intrusive facility
/// implementations, by defining the "true" implementation of the
/// facility to use the single (but type-unsafe) IntrusiveAlias type,
/// while allowing type-safe wrapper implementations to delegate their
/// behavior to the implementation function.
#[derive(PartialEq,Eq,Copy,Clone,Debug)]
#[unstable = "Experimental API"]
pub struct IntrusiveAlias(pub usize);

// FIXME: this wants to be a linear type.
#[unstable = "Experimental API"]
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
    pub unsafe fn into_alias(self) -> IntrusiveAlias {
        let rval = self.pointer;
        mem::forget(self);
        rval
    }
    pub fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias {
        &self.pointer
    }
    pub unsafe fn from_box(b: Box<T>) -> OwnBox<T> {
        OwnBox { pointer: IntrusiveAlias(::std::mem::transmute(b)), marker: marker::PhantomData }
    }
    pub unsafe fn into_box(self) -> Box<T> {
        ::std::mem::transmute(self.into_alias().0)
    }
}
impl<T> ops::Deref for OwnBox<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { ::std::mem::transmute(self.pointer.0) }
    }
}
impl<T> ops::DerefMut for OwnBox<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { ::std::mem::transmute(self.pointer.0) }
    }
}
#[unsafe_destructor]
impl<T> ops::Drop for OwnBox<T> {
    fn drop(&mut self) {
        // should have been consumed via "into_box" or "into_alias".
        panic!("OwnBox should be treated as linear!");
    }
}

#[derive(Debug)]
pub struct BorrowBox<'a, T: 'a> {
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<&'a T>,
}
impl<'a, T> BorrowBox<'a, T> where T: Intrusive {
    pub fn new(source: &'a T) -> BorrowBox<'a, T> {
        unsafe { BorrowBox::new_from(IntrusiveAlias(source as *const T as usize), source) }
    }
    pub unsafe fn new_from<U>(pointer: IntrusiveAlias, _lifetime: &'a U) -> BorrowBox<'a, T> {
        BorrowBox {
            pointer: pointer,
            marker: marker::PhantomData,
        }
    }
    pub fn as_target<'b>(&'b self) -> &'b T where 'b: 'a {
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
        unsafe { BorrowBoxMut::new_from(IntrusiveAlias(source as *mut T as usize), source) }
    }
    pub unsafe fn new_from<U>(pointer: IntrusiveAlias, _lifetime: &'a mut U) -> BorrowBoxMut<'a, T> {
        BorrowBoxMut {
            pointer: pointer,
            marker: marker::PhantomData,
        }
    }
    pub fn as_target<'b>(&'b self) -> &'b T where 'a: 'b {
        unsafe { Intrusive::of_alias(&self.pointer) }
    }
    pub fn as_target_mut<'b>(&'b mut self) -> &'b mut T where 'a: 'b {
        unsafe { Intrusive::of_alias_mut(&mut self.pointer) }
    }
}

/// Trait defining routines for translation between containing
/// structure and intrusive field. The only implementors of this type
/// should be the pointer-types defined by the `containerof_intrusive`
/// macro.
#[unstable = "Experimental API"]
pub trait Intrusive {
    /// Type of containing structure.
    type Container;
    /// Type of intrusive field within containing structure.
    type Field;
    // TODO: would also like to see an "offset" associated const, but
    // Rust doesn't seem to support these yet.

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

    /// Ownership-moving translation from generic intrusive pointer
    /// alias to type-safe intrusive pointer.
    unsafe fn from_alias(IntrusiveAlias) -> Self;

    /// Ownership-moving translation from type-safe intrusive pointer
    /// to generic intrusive pointer.
    unsafe fn into_alias(self) -> IntrusiveAlias;

    /// Allow using type-safe intrusive pointer as generic intrusive pointer.
    unsafe fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias;

    /// Allow using type-safe intrusive pointer as mutable generic
    /// intrusive pointer.
    unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut IntrusiveAlias;

    /// Allow using generic intrusive pointer as type-safe intrusive
    /// pointer.
    unsafe fn of_alias<'a>(&'a IntrusiveAlias) -> &'a Self;

    /// Allow using generic intrusive pointer as mutable type-safe
    /// intrusive pointer.
    unsafe fn of_alias_mut<'a>(&'a mut IntrusiveAlias) -> &'a mut Self;
}
