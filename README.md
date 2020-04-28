logan - Logic Analyzer in Rust
==============================

Install from crates.io
----------------------

This crate is published on https://crates.io/crates/logan to stick to
standard distribution channels, but beware that this is my first
create, and that the repository contains some glue code in different
languages.


Install from source
-------------------

The git repository is at https://github.com/zwizwa/logan

Use `cargo build` to compile the rust code.

Included in the `dev` directory is a C++ wrapper for the Saleae Logic
8 library.  Use `make -C dev` to download upstream library and build
the wrapper.

The `logan` script can be used to start a live analysis session on the
command line.

There is also Erlang code to wrap the `logan` script in
`erl/logan.erl`.  This depends on https://github.com/zwizwa/erl_tools


Status
------

State is proof-of-concept.  There is not yet any documentation but it
is quite straightforward to use and extend if you read Rust.

This started out as a project to try out Rust in a performance
critical setting.  It is a little rough around the edges.  APIs will
probably change slightly to make them more flexible.

This code relies on heavy inlining to get to reasonably good
performance (250-300 M samples/sec on a X201).

