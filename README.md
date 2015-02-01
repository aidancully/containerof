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
* The `containerof_intrusive!` macro, which provides a newtype for
using "intrusive" fields to work with the container object.

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
    fn new() -> Box<Church> {
        Box::new(Church { next: None })
    }
    fn push(next: Box<Church>) -> Box<Church> {
        Box::new(Church { next: Some(unsafe { Intrusive::from_container(next) }) })
    }
    fn pop(self: Box<Church>) -> Option<Box<Church>> {
        match self.next {
            None => None,
            Some(x) => Some(unsafe { x.into_container() }),
        }
    }
}
# fn main() {}
```

# Contributing

1. Fork it ( https://github.com/aidancully/containerof/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request
