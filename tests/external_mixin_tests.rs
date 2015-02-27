#![feature(plugin)]
#![plugin(external_mixin)]

#[test]
fn python() {
    let value = external_mixin! {
        { interpreter = "python" }
        "print(1 + 2)"
    };

    assert_eq!(value, 3);

    let value = python_mixin! {
        "print(1 + 2)"
    };
    assert_eq!(value, 3);
}

#[test]
fn ruby() {
    let value = external_mixin! {
        { interpreter = "ruby" }
        "puts 1 + 2"
    };
    assert_eq!(value, 3);

    let value = ruby_mixin! {
        "puts 1 + 2"
    };
    assert_eq!(value, 3);
}

#[test]
fn sh() {
    let value = external_mixin! {
        { interpreter = "sh" }
        "echo $(expr 1 + 2)"
    };
    assert_eq!(value, 3);

    let value = sh_mixin! {
        "echo $(expr 1 + 2)"
    };
    assert_eq!(value, 3);
}

#[test]
fn perl() {
    let value = external_mixin! {
        { interpreter = "perl" }
        "print 1 + 2"
    };
    assert_eq!(value, 3);

    let value = perl_mixin! {
        "print 1 + 2"
    };
    assert_eq!(value, 3);
}

#[test]
fn args() {
    let value = external_mixin! {
        { interpreter = "sh", arg = "-c", arg = "echo $(expr 1 + 2)" }
        ""
    };
    assert_eq!(value, 3);
}
