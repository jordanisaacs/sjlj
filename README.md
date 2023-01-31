# SJLJ - setjmp & longjmp for Rust

[![crates.io](https://img.shields.io/crates/v/sjlj.svg)](https://crates.io/crates/sjlj)
[![Released API docs](https://docs.rs/sjlj/badge.svg)](https://docs.rs/sjlj)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

A small no_std library that provides inline asm functions for the `setjmp` and `longjmp` functions. Also provides `sigsetjmp` and `siglongjmp` on Linux. This is for using them without requiring a `libc`. The functions are ported from [musl](http://git.musl-libc.org/cgit/musl/tree/src/setjmp).

For a great article on how `setjmp` and `longjmp` work check out Mark Mossberg's [blogpost](https://offlinemark.com/2016/02/09/lets-understand-setjmp-longjmp/) which walks through x86's assembly implementation.

Implemented Architectures:

* `x86_64`
