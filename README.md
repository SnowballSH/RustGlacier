# Glacier Programming Language

Rust implementation of the Glacier 2 programming language.

## History

The Glacier programming language is a successor of my previous programming language, Gorilla.
Gorilla was written in Golang, but due to the implementation's bad memory management and slow speed, I started a new
project called Glacier.

Glacier 1 was merely a clone of Gorilla written in Rust. There is a 3x speed improvement, but still not optimal.

The hope of Glacier 2 is to utilize the best methods for an interpreter. Mark-and-sweep GC, global allocator, simply
bytecodes, and many more are the features I am aiming to use.

## Design

- Simplicity
    - Ruby-style syntax with as little native types as possible.
- Speed
    - Compiled to GlacierVM bytecode and executed by a fast VM.
    - JIT will be implemented in the future.
- Safety
    - No undefined behaviours.
