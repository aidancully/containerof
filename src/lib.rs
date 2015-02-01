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

#[crate_id = "containerof"]
#[crate_type = "lib"]

/// Implement C-like `offsetof` macro in Rust. Will become obsolete
/// when-and-if offsetof() is implemented in the core language.
#[macro_export]
#[unstable = "Will go away if-and-when Rust implements this function directly."]
macro_rules! containerof_field_offset {
    ($container:ty : $field:ident) => (unsafe {
        let nullptr = 0 as * const $container;
        let fieldptr: * const _ = &((*nullptr).$field);
        fieldptr as usize
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
        impl ::containerof::Intrusive for $nt {
            type Container = $container;
            type Field = $fieldtype;

            #[inline]
            unsafe fn from_container(c: Box<$container>) -> Self {
                let cp: *const $container = ::std::mem::transmute(c);
                $nt(::std::mem::transmute(&((*cp).$field)))
            }
            #[inline]
            unsafe fn into_container(self) -> Box<$container> {
                let fieldptr = self.0;
                let containerptr = fieldptr - containerof_field_offset!($container:$field);
                ::std::mem::transmute(containerptr)
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
            unsafe fn from_field(c: Box<$fieldtype>) -> Self {
                $nt(::std::mem::transmute(c))
            }
            #[inline]
            unsafe fn into_field(self) -> Box<$fieldtype> {
                ::std::mem::transmute(self.0)
            }
            #[inline]
            fn as_field<'a>(&'a self) -> &'a $fieldtype {
                unsafe { ::std::mem::transmute(self.0) }
            }
            #[inline]
            fn as_field_mut<'a>(&'a mut self) -> &'a mut $fieldtype {
                unsafe { ::std::mem::transmute(self.0) }
            }

            unsafe fn from_alias(ia: ::containerof::IntrusiveAlias) -> Self {
                ::std::mem::transmute(ia)
            }
            unsafe fn into_alias(self) -> ::containerof::IntrusiveAlias {
                ::std::mem::transmute(self)
            }
            unsafe fn as_alias<'a>(&'a self) -> &'a ::containerof::IntrusiveAlias {
                ::std::mem::transmute(self)
            }
            unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut ::containerof::IntrusiveAlias {
                ::std::mem::transmute(self)
            }
            unsafe fn of_alias<'a>(ia: &'a ::containerof::IntrusiveAlias) -> &'a Self {
                ::std::mem::transmute(ia)
            }
            unsafe fn of_alias_mut<'a>(ia: &'a mut ::containerof::IntrusiveAlias) -> &'a mut Self {
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
#[derive(PartialEq,Eq,Copy,Clone)]
#[unstable = "Experimental API"]
pub struct IntrusiveAlias(pub usize);

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

    // FIXME: I'm not sure that "Box" is correct to use for these
    // "from/into" APIs. Idea is to represent the ownership in a
    // pointer, but does dropping a Box<> pointer generally cause a
    // `free` operation? If so, using these APIs will cause the wrong
    // thing to be done, if the box was not initially obtained via a
    // `malloc`. What I'd really like is an additional LinearBox<> for
    // the `into_field` operation.
    //
    // Since I'm not sure what the correct thing to do is, mark the
    // `ownership` transferring functions as `unsafe`, so that users
    // will know that places where these functions are called will
    // need extra review to ensure correctness. If I get a better idea
    // later, we may be able to make safe versions of these routines.

    /// Represent ownership of a container as ownership of an Intrusive
    /// pointer type. (Inverse of `into_container`.)
    unsafe fn from_container(Box<Self::Container>) -> Self;

    /// Represent ownership of an Intrusive pointer type as ownership of
    /// its container. (Inverse of `from_container`.)
    unsafe fn into_container(self) -> Box<Self::Container>;

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
    unsafe fn from_field(Box<Self::Field>) -> Self;
    
    /// Represent ownership of the container object as ownership of
    /// the intrusive field in the object. (Inverse of `from_field`.)
    unsafe fn into_field(self) -> Box<Self::Field>;

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
