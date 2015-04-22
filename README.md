% containerof - Macros supporting intrusive data structures in Rust.

An intrusive structure is a general-purpose structure directly
embedded within a containing structure, in order to add that
general-purpose facility to the container. As an example, one might
use an intrusive "link" structure to allow objects to be organized in
a linked-list:

```rust
# #[macro_use]
extern crate containerof;
struct Link {
    next: Option<ContainerLink>,
}
struct List {
    head: Option<ContainerLink>,
    tail: Option<ContainerLink>,
}

struct Container {
    link: Link,
}
containerof_intrusive!(ContainerLink = Container:link::Link);
# fn main() {}
```

While this module does not provide a linked-list implementation (for
separation-of-concerns reasons, I believe a linked-list implementation
belongs in a separate crate), it does provide some necessary
abstractions for using intrusive structures:

* The `containerof_field_offset!` macro, which identifies the location
of a field in a containing structure. This isn't too useful in
itself, but is necessary to support:
* The `containerof_intrusive!` macro, which provides a newtype that
describes the translation between the "intrusive" field and the
"container" structure.

# Usage

Here is an example implementation of Church-numerals using an
intrusive linked-list:

```rust
#[macro_use]
extern crate containerof;
use containerof::*;

struct Church {
    next: Option<ChurchLink>,
}

containerof_intrusive!(ChurchLink = Church:next::Option<ChurchLink>);

impl Church {
    fn new() -> OwnBox<Church> {
        unsafe { OwnBox::from_box(Box::new(Church { next: None })) }
    }
    fn push(next: OwnBox<Church>) -> OwnBox<Church> {
        unsafe { OwnBox::from_box(Box::new(Church { next: Some(Intrusive::from_container(next)) })) }
    }
    fn pop(me: OwnBox<Church>) -> Option<OwnBox<Church>> {
        let me = unsafe { me.into_box() };
        match me.next {
            None => None,
            Some(x) => Some(unsafe { x.into_container() }),
        }
    }
}
# fn main() {}
```

# Concepts

`containerof` uses three main concepts for working with intrusive
structures:

1. The intrusive structure itself (`Church.next` in the above example);
2. The containing structure (`Church`);
3. The translation type, for getting a container from a field,
or vice-versa (`ChurchLink`).

In addition, there are three auxiliary structures for managing
ownership and borrowing of intrusive structures:

1. `OwnBox`, which is a pointer type representing ownership of the
container (even if all you have is a field reference).
2. `BorrowBox`, which is a pointer type representing a borrow of the
container.
3. `BorrowBoxMut`, which is a pointer type representing a mutable
borrow of the container.

# Contributing

1. Fork it ( https://github.com/aidancully/containerof/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request
