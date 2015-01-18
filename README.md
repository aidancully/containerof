# rs-intrusive - Support macros for building intrusive data structures
in Rust.

An intrusive structure is a general-purpose structure directly
embedded within a containing structure, in order to add that
general-purpose facility to the container. As an example, one might
use an intrusive "link" structure to allow objects to be organized in
a linked-list:

```rust
struct Link {
    next: Option<Link>,
}
struct List {
    head: Option<Link>,
    tail: Option<Link>,
}

struct Container {
    link: Link,
}
```

While this module does not provide a linked-list implementation (for
separation-of-concerns reasons, I believe a linked-list implementation
belongs in a separate crate), it does provide some necessary
abstractions for using intrusive structures:

* The `field_offset!` macro, which identifies the location of a field
  in a containing structure. This isn't too useful in itself, but is
  necessary to support:
* The `intrusive!` macro, which provides a newtype for using
  "intrusive" fields to work with the container object.

## Usage

```rust
#[macro_use(intrusive)]
extern crate "intrusive";

struct Link {
    next: Option<Link>,
}
// FIXME: is this legal? want to enforce constraint that `List` only
// operates against intrusive `Link`s.
struct<T> List<T> where
    T: Intrusive<Link>,
    T::Field: Link,
{
    head: Option<LinkType>,
}

struct Container {
    link: Link,
}
intrusive!(ContainerLink = Container:link::Link);

fn create() -> List<ContainerLink> {
    let x = Box::new(Container { link: None });
    let link: ContainerLink = intrusive::Intrusive::move_from(x);
    List<ContainerLink> { head: Some(link) }
}
fn head(list: List<ContainerLink>) -> Box<Container> {
    list.head.unwrap().container_of()
}
```

## Contributing

1. Fork it ( https://github.com/aidancully/rs-intrusive/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request
