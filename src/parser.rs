use crate::scanner::{Scanner, Token};
use std::iter::{Iterator, Peekable};

#[derive(Debug, PartialEq)]
pub enum HuckAst<T> { // Boxed to allow data recursion
    Num(u64, T),
    Plus(Box<HuckAst<T>>, Box<HuckAst<T>>, T),
    Minus(Box<HuckAst<T>>, Box<HuckAst<T>>, T),
    Times(Box<HuckAst<T>>, Box<HuckAst<T>>, T),
    Div(Box<HuckAst<T>>, Box<HuckAst<T>>, T),
    Let(String, Box<HuckAst<T>>, T),
    VarRef(String, T),
    Block(Vec<HuckAst<T>>, T),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Eof,
    Fucked(String),
    NotImplemented(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Prec {
    Bottom,
    Expr,
    AddSub,
    MultDiv,
    Top
}

// TODO: better/more idiomatic way of doing this?  possibly look into
// "custom discriminant values for fieldless enumerations" and
// #[repr(u8)]
impl Prec {
    pub fn next(p: Self) -> Self {
        match p {
            Self::Bottom => Self::Expr,
            Self::Expr => Self::AddSub,
            Self::AddSub => Self::MultDiv,
            Self::MultDiv => Self::Top,
            Self::Top => Self::Top,
        }
    }
}

type TokenStream<'a> = Peekable<Scanner<'a>>;

pub type ParseOutput = HuckAst<()>;

type ParseResult = Result<ParseOutput, ParseError>;

type PrefixRule<'a> = fn(&mut Parser<'a>, token: Token<'a>) -> ParseResult;

type InfixRule<'a> = fn(&mut Parser<'a>, token: Token<'a>, lhs: ParseOutput) -> ParseResult;

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenStream<'a>) -> Self {
        Self { tokens }
    }

    pub fn parse(&mut self) -> ParseResult {
        // Start parsing at lowest precendence
        self.parse_prec(Prec::Bottom)
    }

    fn expression(&mut self) -> ParseResult {
        self.parse_prec(Prec::Expr)
    }

    fn parse_prec(&mut self, prec: Prec) -> ParseResult {
        // If we're calling parse_prec, we expect there to be another token
        let t = self.tokens.next().ok_or(ParseError::Eof)?;

        let prefix_rule = Self::get_prefix_rule(t)?;
        let mut lhs = prefix_rule(self, t)?;

        let mut next_prec = self.tokens.peek().map(|next| Self::get_prec(*next));
        // If the next precedence is equal or higher to the current precedence, recur
        while next_prec.is_some() && prec <= next_prec.unwrap() {

            let next = self.tokens.next().unwrap();

            let infix_rule = Self::get_infix_rule(next)?;

            lhs = infix_rule(self, next, lhs)?;
            next_prec = self.tokens.peek().map(|next| Self::get_prec(*next)); // TODO map eta reduce
        }

        Ok(lhs)
    }

    fn number(&mut self, token: Token) -> ParseResult {
        match token {
            Token::Number(num_str) => {
                if let Ok(num) = num_str.parse() {
                    Ok(HuckAst::Num(num, ()))
                } else {
                    Err(ParseError::Fucked(format!("Failed to parse number {}", num_str)))
                }
            }
            _ => Err(ParseError::Fucked(format!("Expected number, found {:?}", token)))
        }
    }

    fn binary(&mut self,
              f: fn (Box<ParseOutput>, Box<ParseOutput>, ()) -> ParseOutput,
              prec: Prec,
              lhs: ParseOutput
    ) -> ParseResult {
        let rhs = self.parse_prec(Prec::next(prec))?;
        Ok(f(Box::new(lhs), Box::new(rhs), ()))
    }

    fn plus(&mut self, _token: Token<'a>, lhs: ParseOutput) -> ParseResult {
        self.binary(HuckAst::Plus, Prec::AddSub, lhs)
    }

    fn minus(&mut self, _token: Token<'a>, lhs: ParseOutput) -> ParseResult {
        self.binary(HuckAst::Minus, Prec::AddSub, lhs)
    }

    fn times(&mut self, _token: Token<'a>, lhs: ParseOutput) -> ParseResult {
        self.binary(HuckAst::Times, Prec::MultDiv, lhs)
    }

    fn div(&mut self, _token: Token<'a>, lhs: ParseOutput) -> ParseResult {
        self.binary(HuckAst::Div, Prec::MultDiv, lhs)
    }

    fn grouping(&mut self, _token: Token<'a>) -> ParseResult {
        let grouping = self.expression()?;
        self.consume(Token::RParen)?;
        Ok(grouping)
    }

    fn block(&mut self, _token: Token<'a>) -> ParseResult {
        let mut exprs = vec![self.expression()?];
        let mut next = self.tokens.peek();

        // There's probably a better way to do this pattern
        while next.is_some() && *next.unwrap() == Token::Semicolon {
            self.consume(Token::Semicolon)?;
            exprs.push(self.expression()?);

            next = self.tokens.peek();
        }
        self.consume(Token::RBrace)?;
        Ok(HuckAst::Block(exprs, ()))
    }

    fn let_decl(&mut self, _token: Token<'a>) -> ParseResult {
        let ident = match self.tokens.next() {
            Some(Token::Var(ident)) => Ok(ident),
            Some(t) => Err(ParseError::Fucked(format!("Expected identifier, found {:?}", t))),
            None => Err(ParseError::Eof),
        }?;

        self.consume(Token::SingleEq)?;

        let expr = self.expression()?;

        Ok(HuckAst::Let(ident.to_string(), Box::new(expr), ()))
    }

    fn var_ref(&mut self, token: Token<'a>) -> ParseResult {
        match token {
            Token::Var(ident) => Ok(HuckAst::VarRef(ident.to_string(), ())),
            _ => Err(ParseError::Fucked("Expected variable reference".to_string()))
        }
    }

    fn consume(&mut self, token: Token) -> Result<(), ParseError> {
        match self.tokens.peek() {
            Some(c) if *c == token => {
                _ = self.tokens.next();
                Ok(())
            },
            _ => Err(ParseError::Fucked(format!("Expected token {:?}", token))),
        }
    }

    fn get_infix_rule(t: Token) -> Result<InfixRule<'a>, ParseError> {
        match t {
            Token::Plus => Ok(Self::plus),
            Token::Minus => Ok(Self::minus),
            Token::Star => Ok(Self::times),
            Token::Slash => Ok(Self::div),
            _ => Err(ParseError::NotImplemented(format!("No infix rule for token type {:?}", t))),
        }
    }
    
    fn get_prefix_rule(t: Token) -> Result<PrefixRule<'a>, ParseError> {
        match t {
            Token::Number(_) => Ok(Self::number),
            Token::LParen => Ok(Self::grouping),
            Token::LBrace => Ok(Self::block),
            Token::Let => Ok(Self::let_decl),
            Token::Var(_) => Ok(Self::var_ref),
            _ => Err(ParseError::NotImplemented(format!("No prefix rule for token type {:?}", t))),
        }
    }

    fn get_prec(t: Token) -> Prec {
        match t {
            Token::Number(_) => Prec::Expr,
            Token::Plus => Prec::AddSub,
            Token::Minus => Prec::AddSub,
            Token::Star => Prec::MultDiv,
            Token::Slash => Prec::MultDiv,
            _ => Prec::Bottom,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::scanner::Scanner;

    use crate::parser::HuckAst::*;

    fn make_scanner(s: &str) -> Peekable<Scanner> {
        Scanner::new(s).peekable()
    }

    #[test]
    fn empty() {
        let scanner = make_scanner("");
        let parsed = Parser::new(scanner).parse();
        assert_eq!(parsed, Err(ParseError::Eof));
    }

    #[test]
    fn number() {
        let scanner = make_scanner("42");
        let parsed = Parser::new(scanner).parse();
        assert_eq!(parsed, Ok(Num(42, ())));
    }

    #[test]
    fn let_decl() {
        let scanner = make_scanner("let var_name = 5");
        let parsed = Parser::new(scanner).parse();
        assert_eq!(parsed, Ok(Let("var_name".to_string(), Box::new(Num(5, ())), ())));
    }

    #[test]
    fn block() {
        let scanner = make_scanner("{let x = 42; x + 1}");
        let parsed = Parser::new(scanner).parse();
        assert_eq!(parsed, Ok(
            Block(vec![
                Let("x".to_string(), Box::new(Num(42, ())), ()),
                Plus(
                    Box::new(VarRef("x".to_string(), ())),
                    Box::new(Num(1, ())),
                    ()
                ),
            ], ())
        ));
    }

    #[test]
    fn arithmetic() {
        let scanner = make_scanner("1 - 2 * 3");
        let parsed = Parser::new(scanner).parse();

        assert_eq!(parsed, Ok(
            Minus(
                Box::new(Num(1, ())),
                Box::new(Times(
                    Box::new(Num(2, ())),
                    Box::new(Num(3, ())),
                    ()
                )),
                ()
            )
        ));
    }

    #[test]
    fn grouping() {
        let scanner = make_scanner("(1 + 2) / 3");
        let parsed = Parser::new(scanner).parse();

        assert_eq!(parsed, Ok(
            Div(
                Box::new(Plus(
                    Box::new(Num(1, ())),
                    Box::new(Num(2, ())),
                    ()
                )),
                Box::new(Num(3, ())),
                ()
            )
        ));
    }

    #[test]
    fn nested_grouping() {
        let scanner = make_scanner("(((420)))");
        assert_eq!(
            Parser::new(scanner).parse(),
            Ok(Num(420, ()))
        )
    }

    #[test]
    fn bad_grouping() {
        let scanner = make_scanner("(2580");
        assert!(Parser::new(scanner).parse().is_err())
    }
}
