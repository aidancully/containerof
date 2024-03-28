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

use std::convert;
use std::marker;
use std::mem;
use std::ops;

#[cfg(has_offset_of)]
/// Implement C-like `offsetof` macro in Rust. Rust has stabilized
/// ::std::mem::offset_of! since 1.77.0, you should use that, instead.
#[macro_export]
macro_rules! containerof_field_offset {
    ($container:ty : $field:ident) => {
        ::std::mem::offset_of!($container, $field)
    };
}

#[cfg(not(has_offset_of))]
/// Implement C-like `offsetof` macro in Rust. Rust has stabilized
/// ::std::mem::offset_of! since 1.77.0, you should use that, instead.
#[macro_export]
macro_rules! containerof_field_offset {
    ($container:ty : $field:ident) => {
        unsafe { &(*(0usize as *const $container)).$field as *const _ as usize }
    };
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
            fn as_alias(&self) -> &IntrusiveAlias {
                unsafe { &*(self as *const _).cast() }
            }
        }
        );
}

/// Alias that has the same representation as an intrusive translation
/// type. The idea is to be able to use this alias for intrusive
/// facility implementations, by defining the "true" implementation of
/// the facility to use the single (but type-unsafe) IntrusiveAlias
/// type, while allowing type-safe wrapper implementations to delegate
/// their behavior to the implementation function.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct IntrusiveAlias(pub *const ());
impl IntrusiveAlias {
    /// Create an IntrusiveAlias instance from a pointer address.
    pub fn new(addr: *const ()) -> IntrusiveAlias {
        IntrusiveAlias(addr)
    }
    /// Create an IntrusiveAlias instance which points to a borrowed
    /// pointer.
    pub fn new_of<T>(addr: &T) -> IntrusiveAlias {
        IntrusiveAlias::new((addr as *const T).cast())
    }
    /// Get back the pointer address from which the `IntrusiveAlias` was
    /// constructed.
    pub fn get_address(&self) -> *const () {
        self.0
    }
}

/// Represent ownership of an object via ownership of an intrusive
/// field within the object. Differs from Rust-standard `Box<T>` in
/// that dropping an `OwnBox<T>` instance is a bug.
// FIXME: this wants to be a linear type, but that requires linear-type
// support in the language.
pub struct OwnBox<T> {
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<T>,
}
impl<T> OwnBox<T> {
    /// Get value pointer address.
    pub fn get_address(&self) -> *const () {
        self.pointer.0
    }
    /// Construct an OwnBox from an IntrusiveAlias pointer.
    /// # Safety
    /// This creates an "owned" structure from a raw pointer, which is
    /// unsafe. The caller must ensure that no copies of `pointer` are
    /// used while `OwnBox` is alive.
    pub unsafe fn from_alias(pointer: IntrusiveAlias) -> OwnBox<T> {
        OwnBox {
            pointer,
            marker: marker::PhantomData,
        }
    }
    /// Move ownership of an OwnBox into an IntrusiveAlias pointer.
    pub fn into_alias(self) -> IntrusiveAlias {
        let rval = self.pointer;
        mem::forget(self);
        rval
    }
    /// Return a borrow-pointer of an IntrusiveAlias with same address
    /// as the OwnBox.
    pub fn as_alias(&self) -> &IntrusiveAlias {
        &self.pointer
    }
    /// Construct an OwnBox from a Box.
    pub fn from_box(b: Box<T>) -> OwnBox<T> {
        OwnBox {
            pointer: IntrusiveAlias::new(unsafe { mem::transmute(b) }),
            marker: marker::PhantomData,
        }
    }
    /// Construct a Box from an OwnBox. Should only be called on an
    /// OwnBox that was constructed via from_box (or
    /// convert::From<Box<_>>).
    /// # Safety
    /// The caller must ensure that `self` was originally constructed
    /// from a `Box`. If it was not, then dropping the resulting Box
    /// will result in an attempt to free an invalid pointer.
    pub unsafe fn into_box(self) -> Box<T> {
        mem::transmute(self.into_alias().get_address())
    }
}
impl<T> convert::From<Box<T>> for OwnBox<T> {
    fn from(t: Box<T>) -> OwnBox<T> {
        OwnBox::from_box(t)
    }
}
impl<T> ops::Deref for OwnBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*(self.get_address() as *const _) }
    }
}
impl<T> ops::DerefMut for OwnBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.get_address() as *mut T) }
    }
}
impl<T> ops::Drop for OwnBox<T> {
    fn drop(&mut self) {
        // should have been consumed via "into_box" or "into_alias".
        // TODO: want a better way to encourage linearity on this type.
        //panic!("containerof::OwnBox should be treated as linear!");
    }
}

/// A borrow-pointer that does not require explicit ownership of the
/// value being borrowed. Used to allow construction of the Intrusive
/// structure translation type from a borrow pointer.
#[derive(Debug)]
pub struct BorrowBox<'a, T: 'a> {
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<&'a T>,
}
impl<'a, T> BorrowBox<'a, T>
where
    T: Intrusive,
{
    /// Build a BorrowBox from a borrow pointer.
    pub fn new(source: &'a T) -> BorrowBox<'a, T> {
        unsafe { BorrowBox::new_from(IntrusiveAlias::new_of(source), source) }
    }
    /// Build a BorrowBox from a raw pointer and a lifetime.
    /// # Safety
    /// This converts a raw pointer to a reference (this type implements
    /// `Deref`). The caller must ensure that the lifetime of the
    /// pointee is outlived by the `_lifetime` argument, and that no
    /// conflicting references are made to the pointee while the
    /// BorrowBox is alive.
    pub unsafe fn new_from<U>(pointer: IntrusiveAlias, _lifetime: &'a U) -> BorrowBox<'a, T> {
        BorrowBox {
            pointer,
            marker: marker::PhantomData,
        }
    }
}
impl<'a, T> ops::Deref for BorrowBox<'a, T>
where
    T: Intrusive,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { Intrusive::of_alias(&self.pointer) }
    }
}

/// A mutable borrow-pointer that does not require explicit ownership
/// of the value being borrowed. Used to allow construction of the
/// Intrusive structure translation type from a mutable borrow
/// pointer.
#[derive(Debug)]
pub struct BorrowBoxMut<'a, T: 'a>
where
    T: Intrusive,
{
    pointer: IntrusiveAlias,
    marker: marker::PhantomData<&'a mut T>,
}
impl<'a, T> BorrowBoxMut<'a, T>
where
    T: Intrusive,
{
    /// Build a BorrowBoxMut from a borrow pointer.
    pub fn new(source: &'a mut T) -> BorrowBoxMut<'a, T> {
        unsafe { BorrowBoxMut::new_from(IntrusiveAlias::new_of(source), source) }
    }
    /// Build a BorrowBoxMut from a raw pointer and a lifetime.
    /// # Safety
    /// This converts a raw pointer to a mutable reference (since this
    /// type implements `Deref` and `DerefMut`). The caller  must ensure
    /// that the lifetime of the pointee is outlived by the `_lifetime`
    /// argument, and that no conflicting references are made to the
    /// pointee while the BorrowBoxMut is alive.
    pub unsafe fn new_from<U>(
        pointer: IntrusiveAlias,
        _lifetime: &'a mut U,
    ) -> BorrowBoxMut<'a, T> {
        BorrowBoxMut {
            pointer,
            marker: marker::PhantomData,
        }
    }
}
impl<'a, T> ops::Deref for BorrowBoxMut<'a, T>
where
    T: Intrusive,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { Intrusive::of_alias(&self.pointer) }
    }
}
impl<'a, T> ops::DerefMut for BorrowBoxMut<'a, T>
where
    T: Intrusive,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { Intrusive::of_alias_mut(&mut self.pointer) }
    }
}

/// Minimal trait that, when implemented for a type, allows for the
/// blanket implementation of the `Intrusive`` trait for that type. This
/// is the trait implemented by the `containerof_intrusive!` macro,
/// and the only implementors of this trait should be the
/// translation-types defined by the `containerof_intrusive!` macro.
pub trait IntrusiveBase: Sized {
    /// Type of containing structure.
    type Container;
    /// Type of intrusive field within containing structure.
    type Field;
    /// Returns offset of intrusive field within containing structure.
    // TODO: would also like to see an "offset" associated const, but
    // Rust doesn't support these yet.
    fn offset() -> usize;

    /// Ownership-moving translation from generic intrusive pointer
    /// alias to type-safe intrusive pointer.
    /// # Safety
    /// The caller must ensure that all uses of the IntrusiveAlias
    /// argument are not used whilt the IntrusiveBase instance is
    /// alive.
    unsafe fn new(ia: IntrusiveAlias) -> Self;

    /// Allow using type-safe intrusive pointer as generic intrusive
    /// pointer.
    fn as_alias(&self) -> &IntrusiveAlias;
}

/// Trait defining routines for translation between containing
/// structure and intrusive field. The only implementors of this trait
/// should be the translation-types defined by the
/// `containerof_intrusive!` macro.
pub trait Intrusive: IntrusiveBase {
    /// Ownership-moving translation from generic intrusive pointer
    /// alias to type-safe intrusive pointer.
    /// # Safety
    /// This converts a raw pointer to an owned pointer, which is unsafe
    /// per Rust's memory model.
    unsafe fn from_alias(ia: IntrusiveAlias) -> Self;

    /// Ownership-moving translation from type-safe intrusive pointer
    /// to generic intrusive pointer.
    /// # Safety
    /// This converts an owned pointer to a raw pointer, which is unsafe
    /// per Rust's memory model.
    unsafe fn into_alias(self) -> IntrusiveAlias;

    /// Allow using type-safe intrusive pointer as mutable generic
    /// intrusive pointer.
    fn as_alias_mut(&mut self) -> &mut IntrusiveAlias;

    /// Allow using generic intrusive pointer as type-safe intrusive
    /// pointer.
    /// # Safety
    /// This converts a raw pointer to a borrowed reference, which is
    /// unsafe per Rust's memory model.
    unsafe fn of_alias(ia: &IntrusiveAlias) -> &Self;

    /// Allow using generic intrusive pointer as mutable type-safe
    /// intrusive pointer.
    /// # Safety
    /// This converts a raw pointer to a mutable borrowed reference,
    /// which is unsafe per Rust's memory model.
    unsafe fn of_alias_mut(ia: &mut IntrusiveAlias) -> &mut Self;

    /// Represent ownership of a container as ownership of an Intrusive
    /// pointer type. (Inverse of `into_container`.)
    fn from_container(c: OwnBox<Self::Container>) -> Self;

    /// Represent ownership of an Intrusive pointer type as ownership of
    /// its container. (Inverse of `from_container`.)
    fn into_container(self) -> OwnBox<Self::Container>;

    /// Represent a borrow of an intrusive type via a borrow of its
    /// container.
    fn of_container(c: &Self::Container) -> BorrowBox<Self>;

    /// Represent a mutable borrow of an intrusive type via a mutable
    /// borrow of its container.
    fn of_container_mut(c: &mut Self::Container) -> BorrowBoxMut<'_, Self>;

    /// Grant referential access to the container of this intrusive
    /// pointer type.
    fn as_container(&self) -> &Self::Container;
    /// Grant mutable referential access to the container of this
    /// intrusive pointer type.
    fn as_container_mut(&mut self) -> &mut Self::Container;

    /// Assuming the "field" is a field in the container object, take
    /// ownership of the field as an intrusive pointer, allowing
    /// eventual translation back to the container. (Inverse of
    /// `into_field`.)
    /// # Safety
    /// The caller must ensure that the `OwnBox` argument was
    /// constructed from this type.
    unsafe fn from_field(c: OwnBox<Self::Field>) -> Self;

    /// Represent ownership of the container object as ownership of
    /// the intrusive field in the object. (Inverse of `from_field`.)
    fn into_field(self) -> OwnBox<Self::Field>;

    /// Represent a borrow of an intrusive type via a borrow of the
    /// intrusive field.
    /// # Safety
    /// The caller must ensure that the `self::Field` argument was
    /// constructed from this type.
    unsafe fn of_field(c: &Self::Field) -> BorrowBox<'_, Self>;

    /// Represent a mutable borrow of an intrusive type via a mutable
    /// borrow of the intrusive field.
    /// # Safety
    /// The caller must ensure that the `self::Field` argument was
    /// constructed from this type.
    unsafe fn of_field_mut(c: &mut Self::Field) -> BorrowBoxMut<'_, Self>;

    /// Grant referential access to the intrusive field represented by
    /// this intrusive pointer.
    fn as_field(&self) -> &Self::Field;

    /// Grant mutable referential access to the intrusive field represented by
    /// this intrusive pointer.
    fn as_field_mut(&mut self) -> &mut Self::Field;
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
    fn as_alias_mut(&mut self) -> &mut IntrusiveAlias {
        unsafe { mem::transmute(self.as_alias()) }
    }
    #[inline]
    unsafe fn of_alias(ia: &IntrusiveAlias) -> &T {
        &*(ia as *const _ as *const T)
    }
    #[inline]
    unsafe fn of_alias_mut(ia: &mut IntrusiveAlias) -> &mut T {
        &mut *(ia as *const _ as *mut T)
    }
    #[inline]
    fn from_container(c: OwnBox<T::Container>) -> Self {
        unsafe {
            let addr: *const u8 = c
                .as_alias()
                .get_address()
                .cast::<u8>()
                .add(<T as IntrusiveBase>::offset());
            mem::forget(c);
            <T as Intrusive>::from_alias(IntrusiveAlias(addr.cast()))
        }
    }
    #[inline]
    fn into_container(self) -> OwnBox<T::Container> {
        unsafe {
            let fieldptr = self.as_alias().get_address().cast::<u8>();
            let containerptr = fieldptr.sub(<T as IntrusiveBase>::offset());
            OwnBox::from_alias(IntrusiveAlias::new(containerptr.cast()))
        }
    }
    #[inline]
    fn of_container(container: &T::Container) -> BorrowBox<'_, T> {
        let addr = (container as *const T::Container).cast::<u8>();
        unsafe {
            let fieldptr = addr.add(<T as IntrusiveBase>::offset());
            BorrowBox::new_from(IntrusiveAlias::new(fieldptr.cast()), container)
        }
    }
    #[inline]
    fn of_container_mut(container: &mut T::Container) -> BorrowBoxMut<'_, T> {
        let addr = (container as *mut T::Container).cast::<u8>();
        unsafe {
            let fieldptr = addr.add(<T as IntrusiveBase>::offset());
            BorrowBoxMut::new_from(IntrusiveAlias::new(fieldptr.cast()), container)
        }
    }
    #[inline]
    fn as_container(&self) -> &T::Container {
        unsafe {
            let fieldptr = self.as_alias().get_address().cast::<u8>();
            let containerptr = fieldptr.sub(<T as IntrusiveBase>::offset());
            &*(containerptr as *const T::Container)
        }
    }
    #[allow(mutable_transmutes)]
    #[inline]
    fn as_container_mut(&mut self) -> &mut T::Container {
        unsafe { mem::transmute(self.as_container()) }
    }
    #[inline]
    unsafe fn from_field(c: OwnBox<T::Field>) -> T {
        let addr = c.as_alias().get_address();
        mem::forget(c);
        <T as Intrusive>::from_alias(IntrusiveAlias::new(addr))
    }
    #[inline]
    fn into_field(self) -> OwnBox<T::Field> {
        unsafe { OwnBox::from_alias(IntrusiveAlias(self.as_alias().get_address())) }
    }
    #[inline]
    unsafe fn of_field(field: &T::Field) -> BorrowBox<'_, T> {
        BorrowBox::new_from(IntrusiveAlias::new_of(field), field)
    }
    #[inline]
    unsafe fn of_field_mut(field: &mut T::Field) -> BorrowBoxMut<'_, T> {
        BorrowBoxMut::new_from(IntrusiveAlias::new_of(field), field)
    }
    #[inline]
    fn as_field(&self) -> &T::Field {
        unsafe { &*(self.as_alias().get_address() as *const _) }
    }
    #[inline]
    fn as_field_mut(&mut self) -> &mut T::Field {
        unsafe { &mut *(self.as_alias().get_address() as *mut _) }
    }
}
