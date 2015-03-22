#![feature(quote, plugin_registrar, rustc_private)]

extern crate "external_mixin_umbrella" as emu;

extern crate syntax;
extern crate rustc;

use std::path::Path;

use syntax::codemap;
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;

use rustc::plugin::Registry;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(registrar: &mut Registry) {
    let plugins = [None,
                   Some("python"),
                   Some("ruby"),
                   Some("sh"),
                   Some("perl")];
    for &interp in &plugins {
        let macro_name = interp.unwrap_or("external");
        let macro_name = format!("{}_mixin", macro_name);
        let macro_name_ = macro_name.clone();
        let expander = move |cx: &mut ExtCtxt, sp, opt, dir: &Path, file: &Path| {
            mixin(&macro_name_, interp,
                  cx, sp, opt, dir, file)
        };

        let interned = token::intern(&macro_name);
        if let Ok(synext) = emu::MixinExpander::new(registrar, macro_name,
                                                    expander) {
            registrar.register_syntax_extension(interned, synext)
        }
    }
}

fn mixin(name: &str,
         interpreter: Option<&str>,
         cx: &mut ExtCtxt, sp: codemap::Span,
         options: emu::Options,
         dir: &Path,
         file: &Path) -> Result<emu::Output, ()> {

    macro_rules! fmt {
        ($fmt_str: tt) => { fmt!($fmt_str, ) };
        ($fmt_str: tt, $($fmt:tt)*) => {
            format!(concat!("`{}!`: ", $fmt_str), name, $($fmt)*)
        }
    }

    let interp = match interpreter {
        Some(s) => s,
        None => {
            match options.get("interpreter") {
                Some(interps) => {
                    assert!(!interps.is_empty());
                    if interps.len() == 1 {
                        &interps[0].0[..]
                    } else {
                        cx.span_err(sp, &fmt!("option `interpreter` specified multiple times"));
                        for &(_, span) in interps {
                            cx.span_note(span, "specified here");
                        }
                        return Err(())
                    }
                }
                None => {
                    cx.span_err(sp, &fmt!("missing option `interpreter`"));
                    return Err(())
                }
            }
        }
    };
    let args: Vec<String> = match options.get("arg") {
        None => vec![],
        Some(args) => args.iter().map(|t| t.0.clone()).collect()
    };

    for (k, vals) in &options {
        match &**k {
            "interpreter" | "arg" => {}
            _ => {
                let span = vals[0].1;
                cx.span_err(span, &fmt!("unknown option `{}`", k));
            }
        }
    }

    emu::run_mixin_command(cx, sp, name, dir, interp, &args, Some(&file))
        .map(emu::Output::Interpreted)
}
