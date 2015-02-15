# la.rs - Logic Analyzer tools for Rust.

This is a spinoff of http://github.com/zwizwa/pyla


# Install

Use `cargo build` to compile just the library exposing reusable code.

Included in `examples` is a C++ driver for Saleae Logic 8.

To use, build a special-purpose stand-alone program with your
configuration hardcoded.  See `examples`.


# Status

Proof-of-concept.  Currently (20150215) only the UART works.  I am
building this to learn how to write performance-critical code in Rust
through a useful, real world example.

This code relies on heavy inlining to get to reasonably good
performance (250-300 M samples/sec on a X201).  
