#![feature(quote, plugin_registrar, rustc_private)]
#![feature(tempdir, std_misc)]

extern crate syntax;
extern crate rustc;

use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::fs::{self, TempDir, File};
use std::path::{PathBuf, Path};
use std::process;

use syntax::ast;
use syntax::codemap;
use syntax::ext::base::{self, ExtCtxt, MacResult};
use syntax::fold::Folder;
use syntax::parse::{self, token};
use rustc::plugin::Registry;

mod parser_any_macro;

pub fn run_mixin_command(cx: &ExtCtxt, sp: codemap::Span,
                         plugin_name: &str, dir: &Path, binary: &str, args: &[String],
                         file: Option<&Path>) -> Result<process::Output, ()> {

    let mut command = process::Command::new(&binary);
    command.current_dir(dir)
        .args(&args);

    if let Some(file) = file {
        command.arg(&file);
    }

    match command.output() {
        Ok(o) => Ok(o),
        Err(e) => {
            let msg = if args.is_empty() {
                format!("`{}!`: could not execute `{}`: {}", plugin_name, binary, e)
            } else {
                format!("`{}!`: could not execute `{}` with argument{} `{}`: {}",
                        plugin_name,
                        binary,
                        if args.len() == 1 { "" } else {"s"},
                        args.connect("`, `"),
                        e)
            };
            cx.span_err(sp, &msg);
            Err(())
        }
    }
}


/// The options passed to the macro.
pub type Options = HashMap<String, Vec<(String, codemap::Span)>>;

pub enum Output {
    Interpreted(process::Output),
    Compiled(process::Output, String),
}

///
pub struct MixinExpander<F>
    where F: Fn(&mut ExtCtxt, codemap::Span,
                Options,
                &Path,
                &Path) -> Result<Output, ()>
{
    name: String,
    dir: TempDir,
    expander: F
}

impl<F> MixinExpander<F>
    where F: 'static + Fn(&mut ExtCtxt, codemap::Span,
                          Options,
                          &Path,
                          &Path) -> Result<Output, ()>
{
    pub fn new(reg: &Registry, name: String, expander: F)
               -> Result<base::SyntaxExtension, ()> {
        let dir =  match TempDir::new(&format!("rustc_external_mixin_{}", name)) {
            Ok(d) => d,
            Err(e) => {
                reg.sess.span_err(reg.krate_span,
                                  &format!("`{}!`: could not create temporary directory: {}",
                                           name,
                                           e));
                return Err(())
            }
        };

        Ok(base::NormalTT(Box::new(MixinExpander {
            name: name,
            dir: dir,
            expander: expander
        }), None, false))
    }
    fn handle(&self, cx: &ExtCtxt, sp: codemap::Span,
              output: Output) -> Result<Vec<u8>, ()> {
        match output {
            Output::Interpreted(out) => {
                try!(check_errors_raw(cx, sp, &self.name, "code", &out));
                Ok(out.stdout)
            }
            Output::Compiled(out, exe) => {
                try!(check_errors_raw(cx, sp, &self.name, "compiler", &out));
                let out = try!(run_mixin_command(cx, sp,
                                                 &self.name, self.dir.path(), &exe, &[], None));
                try!(check_errors_raw(cx, sp, &self.name, "binary", &out));
                Ok(out.stdout)
            }
        }
    }
}

fn check_errors_raw(cx: &ExtCtxt,sp: codemap::Span,
                    name: &str,
                    kind: &str,
                    output: &process::Output) -> Result<(), ()> {
    if !output.status.success() {
        cx.span_err(sp,
                    &format!("`{}!`: the {} did not execute successfully: {}",
                             name, kind, output.status));
        let msg = if output.stderr.is_empty() {
            "there was no output on stderr".to_string()
        } else {
            format!("the {} emitted the following on stderr:\n{}",
                    kind, String::from_utf8_lossy(&output.stderr))
        };
        cx.parse_sess().span_diagnostic.fileline_note(sp, &msg);
        return Err(())
    } else if !output.stderr.is_empty() {
        cx.span_warn(sp, &format!("`{}!`: the {} ran successfully, but had output on stderr",
                                  name, kind));
        let msg = format!("output:\n{}",
                          String::from_utf8_lossy(&output.stderr));
        cx.parse_sess().span_diagnostic.fileline_note(sp, &msg);
    }
    Ok(())
}




impl<F> base::TTMacroExpander for MixinExpander<F>
    where F: 'static + Fn(&mut ExtCtxt, codemap::Span,
                          Options,
                          &Path,
                          &Path) -> Result<Output, ()>

{
    fn expand<'cx>(&self,
                   cx: &'cx mut ExtCtxt,
                   sp: codemap::Span,
                   raw_tts: &[ast::TokenTree])
                   -> Box<MacResult+'cx>
    {
        macro_rules! mac_try {
            ($run: expr, $p: pat => $fmt_str: tt, $($fmt: tt)*) => {
                match $run {
                    Ok(x) => x,
                    Err($p) => {
                        cx.span_err(sp,
                                    &format!(concat!("`{}!`: ", $fmt_str), self.name, $($fmt)*));
                        return base::DummyResult::any(sp)
                    }
                }
            }
        }
        let (tts, option_tts): (_, &[_]) = match raw_tts.get(0) {
            Some(&ast::TtDelimited(_, ref delim)) if delim.delim == token::Brace => {
                (&raw_tts[1..], &delim.tts[..])
            }
            _ => (raw_tts, &[])
        };

        let options = match parse_options(cx, option_tts) {
            Ok(o) => o,
            Err(_) => return base::DummyResult::any(sp)
        };

        let code = match base::get_single_str_from_tts(cx, sp, tts,
                                                       &format!("`{}!`", self.name)) {
            Some(c) => c,
            None => return base::DummyResult::any(sp)
        };

        let lo = tts[0].get_span().lo;
        let first_line = cx.codemap().lookup_char_pos(lo).line as u64;

        let filename = cx.codemap().span_to_filename(sp);
        let path = PathBuf::new(&filename);
        let name = if path.is_absolute() {
            Path::new(path.file_name().unwrap())
        } else {
            &*path
        };
        let code_file = self.dir.path().join(name);

        mac_try! {
            fs::create_dir_all(code_file.parent().unwrap()),
            e => "could not create temporary directory: {}", e
        }

        let file = mac_try!(File::create(&code_file),
                            e => "could not create temporary file: {}", e);
        let mut file = io::BufWriter::new(file);

        mac_try!(io::copy(&mut io::repeat(b'\n').take(first_line - 1), &mut file),
                 e => "could not write output: {}", e);
        mac_try!(file.write(code.as_bytes()),
                 e => "could not write output: {}", e);
        mac_try!(file.flush(),
                 e => "could not flush output: {}", e);
        drop(file);

        let output = (self.expander)(cx, sp, options, self.dir.path(), &name)
            .and_then(|o| self.handle(cx, sp, o));

        let output = match output {
            Ok(o) => o,
            // handled internally...
            Err(_) => return base::DummyResult::any(sp)
        };

        let emitted_code = mac_try!(String::from_utf8(output),
                                    e => "emitted invalid UTF-8: {}", e);

        let name = format!("<{}:{} {}!>", filename, first_line, self.name);
        let parser = parse::new_parser_from_source_str(cx.parse_sess(),
                                                       cx.cfg(),
                                                       name,
                                                       emitted_code);

        Box::new(parser_any_macro::ParserAnyMacro::new(parser))
    }
}

fn parse_options(cx: &mut ExtCtxt, option_tts: &[ast::TokenTree]) -> Result<Options, ()> {
    let mut error_occurred = false;

    let mut options = HashMap::new();
    let mut p = cx.new_parser_from_tts(option_tts);
    while p.token != token::Eof {
        // <name> = "..."
        let ident = p.parse_ident();
        let ident_span = p.last_span;
        let key = ident.as_str().to_string();
        p.expect(&token::Eq);

        let ret = cx.expander().fold_expr(p.parse_expr());
        let span = codemap::mk_sp(ident_span.lo, ret.span.hi);

        let val_opt = base::expr_to_string(cx, ret, "option must be a string literal")
            .map(|(s, _)| s.to_string());
        let val = match val_opt {
            None => {
                error_occurred = true;
                while p.token != token::Comma { p.bump() }
                continue
            }
            Some(v) => v
        };


        options.entry(key)
            .get()
            .unwrap_or_else(|v| v.insert(vec![]))
            .push((val, span));

        if p.token == token::Eof { break }
        p.expect(&token::Comma);
    }

    if error_occurred {
        Err(())
    } else {
        Ok(options)
    }
}
