# Overview

A couple of basic implementations of an arbitrary precision integer in Rust.

Only positive integers are supported and only sum and product are implemented.

## easy

Numbers are represented as binary integers and stored in an array of `u8` (one digit per byte).

Parsing from a string containing a number in binary format is implemented.

## optimized_memory

Numbers are represented as integers in base 2^32 and stored in an array of `u32`
(hence memory usage is optimal).

Parsing from a string containing a number in decimal format is implemented.
