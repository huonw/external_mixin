#![feature(plugin)]
#![plugin(rust_mixin)]

#[cfg(feature = "compile_error")]
fn foo() {
    rust_mixin! {
        {arg = "-bad-argument"}
        ""
    }

    // bad syntax (hopefully the rust errors point to the bad line)
    rust_mixin! {"
invalid invalid
    "}

    // successful run, but output on stderr
    rust_mixin! {
        r#"
#![feature(old_io)]
use std::old_io;
fn main() {
    println!("()");
    writeln!(&mut old_io::stderr(), "this is on stderr").unwrap();
}
    "#}

    // uncaught panic
    rust_mixin! {"fn main() { panic!() }"}
    // segfault!
    rust_mixin! {r#"
fn main() {
    let _x = unsafe {*(1 as *const u8)};
}
    "#}
}

fn main() {}
