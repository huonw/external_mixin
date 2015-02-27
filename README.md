# Mixins for Rust

[![Build Status](https://travis-ci.org/huonw/external_mixin.png)](https://travis-ci.org/huonw/external_mixin)

Write code in arbitrary languages, to emit Rust code right in your
crate.

```rust
#![feature(plugin)]
#![plugin(external_mixin)]
#![plugin(rust_mixin)]

python_mixin! {"
x = 1 + 2
print('fn get_x() -> u64 { %d }' % x)
"}

fn main() {
    let value = get_x();

    let other_value = rust_mixin! {r#"
fn main() {
    println!("{}", 3 + 4);
}
    "#};

    assert_eq!(value, 3);
    assert_eq!(other_value, 7);
}
```

This comes in three libraries:

- [`rust_mixin`](#rust_mixin) — use Rust to generate your code.
- [`external_mixin`](#external_mixin) — use scripting languages like
  Python or Ruby to generate your code.
- [`external_mixin_umbrella`](#external_mixin_umbrella) — support
  library, to keep the above DRY.

## Installation

Both plugin crates are available on crates.io:
[`rust_mixin`](https://crates.io/crates/rust_mixin),
[`external_mixin`](https://crates.io/crates/external_mixin). Hence,
you can add any subset of

```toml
[dependencies]
rust_mixin = "*"
external_mixin = "*"
```

to your `Cargo.toml`.


## `rust_mixin`

Write Rust to generate your Rust, right in your Rust (yo dawg). The
plugin compiles and runs its argument as a Rust program, and then
inserts the output into the main crate, similar to a `macro_rules!`
macro.

The `rust_mixin` plugin takes a single string, containing a Rust
program to be compiled with `rustc`. This program should print valid
Rust to stdout. Each `rust_mixin` invocation is independent of all
others. The string argument is macro-expanded before being used, so
constructing an invocation with `concat!()` is legitimate.

The macro supports an optional `{ ... }` block before the string
literal, to specify options. The only option supported currently is
`arg`: it can be specified multiple times, and the arguments are
passed to `rustc` in the order given.

This doesn't currently support using any dependencies via `cargo`.

### Examples

Compute Fibonacci numbers in the *best* way possible, by making Rust
print a function to compute each number:

```rust
#![feature(plugin)]
#![plugin(rust_mixin)]

rust_mixin! {r#"
fn main() {
    println!("fn fib_0() -> i32 {{ 0 }}");
    println!("fn fib_1() -> i32 {{ 1 }}");

    for i in 2..(40 + 1) {
        println!("fn fib_{}() -> i32 {{ fib_{}() + fib_{}() }}",
                 i, i - 1, i - 2);
    }
}
"#}

fn main() {
    println!("the 30th fibonacci number is {}", fib_30());
}
```

Do the Fibonacci computation at compile time, naively, so we want some
optimisations:

```rust
#![feature(plugin)]
#![plugin(rust_mixin)]

fn main() {
    let fib_30 = rust_mixin! {
        { arg = "-C", arg = "opt-level=3" }
        r#"
fn fib(n: u64) -> u64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}
fn main() {
    println!("{}", fib(30))
}
    "#};


    println!("the 30th fibonacci number is {}", fib_30);
}
```

## `external_mixin`

Use a variety of scripting languages to generate Rust code. This has
an `external_mixin!` macro that supports arbitrary interpreters, as
well as specialise support for several languages: `python_mixin!`,
`ruby_mixin!`, `sh_mixin!`, `perl_mixin!`.

As with `rust_mixin!` these macros take their program as a string that
gets macro expanded, and each invocation is independent of all
others. The program should print valid Rust to stdout. Options can be
specified with an optional `{ ... }` block, before the string literal.

The `external_mixin!` macro is the most flexible form, it takes a
compulsory `interpreter` argument: this program is called with a file
containing the code snippet as the last argument.

Both `external_mixin!` and the language specific macros support the
`arg` option, which can be specified multiple times and are passed to
the main binary, in the order given.

### Portability?

These macros rely on shelling out to interpreters, relying on there
being an appropriately named executable in the user's path (hopefully
it is the right version, too...). Hence, this is not portable or
reliable. At least a user of `rust_mixin!` is guarantee to have a
`rustc` available, no such guarantee exists here.

### Examples

Count how many files/folders lie at the top of the (Unix) file system.

```rust
#![feature(plugin)]
#![plugin(external_mixin)]

fn main() {
    let file_count = sh_mixin!("ls / | wc -l");
    println!("there are {} files in /", file_count);
}
```

Compute the Unix time that the program was built at, via Ruby.

```rust
#![feature(plugin)]
#![plugin(external_mixin)]

fn main() {
    let build_time = ruby_mixin!("puts Time.now.to_i");
    println!("this was built {} seconds after 1970-01-01 00:00:00", build_time);
}
```

Use Python 2's naked print statement and Python 3's division semantics
(and guess the version of the `python` binary, used by
`python_mixin!`):

```rust
#![feature(plugin)]
#![plugin(external_mixin)]

fn main() {
    let value2 = external_mixin! {
        { interpreter = "python2" }
        "print 1 / 2"
    };
    let value3 = external_mixin! {
        { interpreter = "python3" }
        "print(1 / 2)"
    };
    let value_unknown = python_mixin!("print(1 / 2)");

    if value_unknown as f64 == value3 {
        println!("`python_mixin!` is Python 3");
    } else {
        println!("`python_mixin!` is Python 2");
    }
}
```

## `external_mixin_umbrella`

The top level item of this repository is a library designed to
maximise the sharing of code between `external_mixin` and
`rust_mixin`, so that their implementations are only 100 and 50 lines
respectively.

## All your questions... answered:

## Should I actually use these?

Probably not, this is me experimenting with
[more](https://github.com/huonw/brainfuck_macros) language
[plugins](https://github.com/huonw/fractran_macros). A more
portable/usable way to do this sort of code-generation is via
[a `Cargo` build script](http://doc.crates.io/build-script.html) plus
the `include!` macro.

Some downsides (not exhaustive):

- the mixins like `python_mixin!` rely on having correctly-named
  binaries in the user's path, and, e.g. "`python`" is sometimes
  Python 2 and sometimes Python 3. Also, it's mean to require users to
  have installed Python on Windows. (Build scripts only need a Cargo
  and a Rust compiler, which the user is guaranteed to have if they're
  trying to build your Rust code.)

- errors in the generated code are hard to debug, although the macros
  do try to give as useful error messages as possible e.g. file/line
  numbers for errors in the code point as closely as possible to the
  relevant part of the original string containing the source
  (including working with editors' jump-to-error facilities). However,
  the parsed Rust doesn't actually appear anywhere on disk or
  otherwise, so you cannot easily see the full context when the
  compiler complains (in contrast, a build script just generates a
  normal file right in your file-system).

### Why not use token trees, rather than strings?

It doesn't work so well for the white-space sensitive languages, and
the arbitrary other languages usable with `external_mixin` can have
wildly different syntax to Rust, syntax that doesn't even tokenise as
Rust, e.g. `'foo'` is a valid string in many scripting languages, but
is an invalid character literal in Rust. Rust's `#`-delimited raw
strings means that there is no escaping required (just add enough
`#`s, usually one is all that is needed).

For `rust_mixin!`, I thought consistency was nice, and it provides a
distinction between the real program and the subprogram via syntax
highlighting (having two `main`s in one file is confusing).
