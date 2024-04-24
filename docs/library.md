# DRust Library

To ensure robustness and maintainability, DRust is provided as a framework designed to help developers easily distribute their applications across multiple servers, with consistency and high efficiency. DRust's library includes functionalities akin to many standard Rust constructs, providing familiar interfaces while supporting distributed settings.

## Getting Started

To deploy your own application on DRust, add your application code to the `drust/src/app` directory and rename your `main` function to `run`. Additionally, you will need to rename some basic language constructs to DRust equivalents (e.g., `Box` to `DBox`, `Vec` to `DVec`). This process should be straightforward, and a tool will be provided to facilitate it. DRust also provides four example applications to show the ability and potential of DRust. 

DRust currently supports remote async thread spawning, so applications must be structured to fit this model.

## DRust Language Constructs


### DBox

In Rust, the `Box` type is a smart pointer that provides heap allocation for storing data. It is a way to allocate values on the heap instead of the stack. DRust's `DBox` serves the same purpose, but with additional support for remote heap data store. When dereferencing a `DBox`, data can be automatically migrated from a remote server to local memory. `DBox` has the same interface as `Box`. For example:

```rust
let v = DBox::new(1);
println!("Value of v: {}", *v);
```

But data owned by the pointer could be located at a remote server. When derefencing the Box pointer, the data would be automatically migrated to local memory.

### DRust Vector

DRust introduces `DVec`, a distributed vector that extends Rust's standard `Vec` with distributed-memory features. Unlike traditional `Vec`, which stores all data on a single server, `DVec` allows for data to be distributed across multiple servers. Despite this, `DVec` behaves much like `Vec`, offering similar methods and functionality.

#### Creating and Using DVec

You can create and manipulate `DVec` just as you would with `Vec`. Here's an example that demonstrates basic usage:

```rust
let v = DVec::with_capacity(100);
v.push(1);
println!("Length of v: {}, v[0]: {}", v.len(), v[0]);
```

### Remote Thread Spawning

DRust also introduces remote thread spawning with the `dspawn` function, similar to Rust's `tokio::spawn`. This feature allows you to create asynchronous threads on different servers, potentially improving scalability and resource utilization. The location of the spawned thread is automatically chosen based on current workload and resource availability. If you want more control over where the remote thread is spawned, DRust offers variants like `dspawn_to`, allowing you to specify the target server or resource. This can be useful when you need to balance loads or ensure specific hardware is utilized.

#### Using dspawn for Remote Threads

Here's an example demonstrating how to use dspawn to execute a function remotely and then retrieve the result:

```rust
async fn remote_function(x: usize) -> usize {
    x * x
}

let handle = dspawn(remote_function(3)); // Spawn the function remotely
let result = handle.await.unwrap();       // Await the result
println!("Value of square of 3: {}", result); // Output: Value of square of 3: 9
```


Additional documentation for other types (e.g., `TBox`, `DString`, `DRef`, `DMut`) will be provided later.
