#![feature(plugin)]
#![plugin(external_mixin)]

#[cfg(feature = "compile_error")]
fn foo() {
    // option double up
    external_mixin! {
        { interpreter = "foo", interpreter = "bar" }
        ""
    }
    python_mixin! {
        {arg = "-bad-argument"}
        ""
    }

    // bad syntax (hopefully the python errors point to the bad line)
    python_mixin! {"
invalid invalid
    "}

    // successful run, but output on stderr
    python_mixin! {
        r#"
import sys
print('()')
sys.stderr.write("this is on stderr")
    "#}

    // uncaught exception
    python_mixin! {"raise ValueError()"}
    // segfault!
    python_mixin! {r#"
import ctypes
ctypes.string_at(0)
    "#}
}

fn main() {}
