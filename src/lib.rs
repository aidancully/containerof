#![crate_name = "containerof"]

// offsetof()-like operation. Will become obsolete when-and-if offsetof() is
// implemented in the core language.
#[macro_export]    
macro_rules! containerof_field_offset {
    ($container:ty : $field:ident) => (unsafe {
        let nullptr = 0 as * const $container;
        let fieldptr: * const _ = &((*nullptr).$field);
        fieldptr as usize
    })
}

#[macro_export]
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
    )
}

// Defined in the same way as the `newtype` formed by the intrusive
// macro, *should* have the same representation. (FIXME: can this be
// guaranteed, somehow? Right now, we rely on std::mem::transmute's
// promise not to compile when provided an incompatible data type, but
// I'm still nervous about this.) Intended to be provided as an input
// to `Intrusive`'s `*_alias` routines, making it easier for
// implementors of intrusive structures to use common implementations.
#[derive(PartialEq,Eq,Copy,Clone)]
pub struct IntrusiveAlias(pub usize);

pub trait Intrusive {
    type Container;
    type Field;

    // FIXME: I'm not sure that "Box" is correct to use for these
    // "from/to" APIs. Idea is to represent the ownership in a
    // pointer, but does dropping a Box<> pointer generally cause a
    // `free` operation? If so, using these APIs may cause the wrong
    // thing to be done, if the box was not initially obtained via a
    // `malloc`.
    //
    // Since I'm not sure what the correct thing to do is, mark the
    // `ownership` transferring functions as `unsafe`, so that users
    // will know that places where these functions are called will
    // need extra review to ensure correctness. If I get a better idea
    // later, we may be able to make safe versions of these routines.

    unsafe fn from_container(Box<Self::Container>) -> Self;
    unsafe fn into_container(self) -> Box<Self::Container>;
    fn as_container<'a>(&'a self) -> &'a Self::Container;
    fn as_container_mut<'a>(&'a mut self) -> &'a mut Self::Container;

    unsafe fn from_field(Box<Self::Field>) -> Self;
    unsafe fn into_field(self) -> Box<Self::Field>;
    fn as_field<'a>(&'a self) -> &'a Self::Field;
    fn as_field_mut<'a>(&'a mut self) -> &'a mut Self::Field;

    unsafe fn from_alias(IntrusiveAlias) -> Self;
    unsafe fn into_alias(self) -> IntrusiveAlias;
    unsafe fn as_alias<'a>(&'a self) -> &'a IntrusiveAlias;
    unsafe fn as_alias_mut<'a>(&'a mut self) -> &'a mut IntrusiveAlias;
    unsafe fn of_alias<'a>(&'a IntrusiveAlias) -> &'a Self;
    unsafe fn of_alias_mut<'a>(&'a mut IntrusiveAlias) -> &'a mut Self;
}
