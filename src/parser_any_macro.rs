// from src/libsyntax/ext/tt/macro_rules.rs

use std::cell::RefCell;
use syntax::parse::parser::Parser;
use syntax::parse::token;
use syntax::ast;
use syntax::ptr::P;
use syntax::ext::base::MacResult;
use syntax::util::small_vector::SmallVector;

pub struct ParserAnyMacro<'a> {
    parser: RefCell<Parser<'a>>,
}

impl<'a> ParserAnyMacro<'a> {
    pub fn new(p: Parser<'a>) -> ParserAnyMacro<'a> {
        ParserAnyMacro {
            parser: RefCell::new(p)
        }
    }
    /// Make sure we don't have any tokens left to parse, so we don't
    /// silently drop anything. `allow_semi` is so that "optional"
    /// semicolons at the end of normal expressions aren't complained
    /// about e.g. the semicolon in `macro_rules! kapow { () => {
    /// panic!(); } }` doesn't get picked up by .parse_expr(), but it's
    /// allowed to be there.
    fn ensure_complete_parse(&self, allow_semi: bool) {
        let mut parser = self.parser.borrow_mut();
        if allow_semi && parser.token == token::Semi {
            parser.bump()
        }
        if parser.token != token::Eof {
            let token_str = parser.this_token_to_string();
            let msg = format!("macro expansion ignores token `{}` and any \
                               following",
                              token_str);
            let span = parser.span;
            parser.span_err(span, &msg[..]);
        }
    }
}

impl<'a> MacResult for ParserAnyMacro<'a> {
    fn make_expr(self: Box<ParserAnyMacro<'a>>) -> Option<P<ast::Expr>> {
        let ret = self.parser.borrow_mut().parse_expr();
        self.ensure_complete_parse(true);
        Some(ret)
    }
    fn make_pat(self: Box<ParserAnyMacro<'a>>) -> Option<P<ast::Pat>> {
        let ret = self.parser.borrow_mut().parse_pat();
        self.ensure_complete_parse(false);
        Some(ret)
    }
    fn make_items(self: Box<ParserAnyMacro<'a>>) -> Option<SmallVector<P<ast::Item>>> {
        let mut ret = SmallVector::zero();
        while let Some(item) = self.parser.borrow_mut().parse_item() {
            ret.push(item);
        }
        self.ensure_complete_parse(false);
        Some(ret)
    }

    fn make_impl_items(self: Box<ParserAnyMacro<'a>>)
                       -> Option<SmallVector<P<ast::ImplItem>>> {
        let mut ret = SmallVector::zero();
        loop {
            let mut parser = self.parser.borrow_mut();
            match parser.token {
                token::Eof => break,
                _ => ret.push(parser.parse_impl_item())
            }
        }
        self.ensure_complete_parse(false);
        Some(ret)
    }

    fn make_stmt(self: Box<ParserAnyMacro<'a>>) -> Option<P<ast::Stmt>> {
        let ret = self.parser.borrow_mut().parse_stmt();
        self.ensure_complete_parse(true);
        ret
    }
}
