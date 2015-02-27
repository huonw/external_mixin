#![feature(plugin)]
#![plugin(rust_mixin)]

#[test]
fn smoke() {
    let value = rust_mixin! {r#"
fn main() {
    println!("{}", 1 + 2);
}
    "#};

    assert_eq!(value, 3);
}

#[test]
fn args() {
    rust_mixin! {
        // this lint is on deny by default, so it needs an
        // explicit allow or this won't compile.
        { arg = "--allow=exceeding_bitshifts" }
        r#"
#[allow(dead_code)]
fn foo() { 1u8 << 1000; }
fn main() { println!("()")}
"#
    };
}
