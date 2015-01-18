#![crate_name = "intrusive"]

// offsetof()-like operation. Will become obsolete when-and-if offsetof() is
// implemented in the core language.
#[macro_export]    
macro_rules! field_offset {
    // FIXME: I'm having a hard time figuring out how to make this
    // hygienic.
    ($container:ty : $field:ident) => (unsafe {
        let nullptr = 0 as * const $container;
        let fieldptr: * const _ = &((*nullptr).$field);
        fieldptr as usize
    })
}

#[macro_export]    
macro_rules! intrusive {
    ($nt:ident = $container:ty : $field:ident :: $fieldtype:ty) => (
        // FIXME: $nt should really be a linear type (it is an error to
        // drop an instance of $nt, as the container needs to be recovered
        // for drop to succeed, so drops should be prevented at
        // compiler-level), but Rust yet doesn't support linear types.
        struct $nt(usize);
        impl ::intrusive::Intrusive for $nt {
            type Container = $container;
            type Field = $fieldtype;

            #[inline]
            fn move_from(c: Box<$container>) -> Self {
                unsafe {
                    let cp: *const $container = ::std::mem::transmute(c);
                    $nt(::std::mem::transmute(&((*cp).$field)))
                }
            }
            #[inline]
            fn container_of(self) -> Box<$container> {
                unsafe {
                    let fieldptr: usize = ::std::mem::transmute(self.0);
                    let containerptr =
                        fieldptr - field_offset!($container:$field);
                    ::std::mem::transmute(containerptr)
                }
            }
            #[inline]
            fn as_field(&self) -> &$fieldtype {
                unsafe {
                    ::std::mem::transmute(self.0)
                }
            }
            #[inline]
            fn as_field_mut(&mut self) -> &mut $fieldtype {
                unsafe {
                    ::std::mem::transmute(self.0)
                }
            }
        }
    )
}

pub trait Intrusive {
    type Container;
    type Field;

    fn move_from(Box<Self::Container>) -> Self;
    fn container_of(self) -> Box<Self::Container>;
    fn as_field(&self) -> &Self::Field;
    fn as_field_mut(&mut self) -> &mut Self::Field;
}
