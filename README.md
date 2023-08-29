# type-handle

Tiny Rust library that exports `Handle<T>` and `RCHandle<T>`. Can be useful for wrapping native ffi structs/pointers. 

Both `Handle` and `RCHandle` implement `Clone`, where `Handle` will clone the underlying struct instance (if it implements `Clone`), and `RCHandle` will keep the underlying pointer.

`Handle` and `RCHandle` implement `Send`/`Sync` by default with the `send_sync` feature.

They both implement `Deref` and `DerefMut`, so you can access a field through a handle the same as you normally would on a normal instance.

# Example

## `Handle<T>`

```rust
#[derive(Clone)]
struct Animal {
    is_dog: bool,
}

let animal = Animal { is_dog: false };
let cat = Handle::from_instance(animal);

// clone `cat` and mutate `is_dog`, note that `animal` is not mutable
let dog = handle.clone(); // this clones `Animal`, `Animal` must implement `Clone`
dog.is_dog = true;
```

## `RCHandle<T>` (reference-counted handle)

```rust
// don't have to #[derive(Clone)] here!
struct Animal {
    is_dog: bool,
}

let mut animal = Animal { is_dog: false };
let mut handle = RCHandle::from_ptr(&mut animal);

// note that `Animal` does not implement `Clone`, because 
// cloning an `RCHandle` does not clone the underlying type
let mut handle2 = handle.clone();

handle2.is_dog = true;
assert!(handle.is_dog == handle2.is_dog);
```

# Tests

To run tests, run `cargo test`.

# License

Public domain (unlicense).