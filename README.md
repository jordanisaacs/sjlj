# SJLJ - shortjmp & longjmp for Rust

A small library that provides inline asm functions for the `shortjmp` and `longjmp` functions. This is for using them without requiring a `libc`. The assembly is translated into intel syntax from [musl](http://git.musl-libc.org/cgit/musl/tree/src/setjmp).

For a great article on how `shortjmp` and `longjmp` work check out Mark Mossberg's [blogpost](https://offlinemark.com/2016/02/09/lets-understand-setjmp-longjmp/) which walks through x86's assembly implementation.

Implemented Architectures:

* `x86_64`
