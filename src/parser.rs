use crate::scanner::{Scanner, Token};
use std::iter::{Iterator, Peekable};

#[derive(Debug, PartialEq)]
pub enum HuckAst { // Boxed to allow data recursion
    Num(u64),
    Plus(Box<HuckAst>, Box<HuckAst>),
    Minus(Box<HuckAst>, Box<HuckAst>),
    Times(Box<HuckAst>, Box<HuckAst>),
    Div(Box<HuckAst>, Box<HuckAst>),
//    Let(String, Box<HuckAst>),
    Block(Vec<HuckAst>),
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

type ParseResult = Result<HuckAst, ParseError>;

type PrefixRule<'a> = fn(&mut Parser<'a>, token: Token<'a>) -> ParseResult;

type InfixRule<'a> = fn(&mut Parser<'a>, token: Token<'a>, lhs: HuckAst) -> ParseResult;

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
                    Ok(HuckAst::Num(num))
                } else {
                    Err(ParseError::Fucked(format!("Failed to parse number {}", num_str)))
                }
            }
            _ => Err(ParseError::Fucked(format!("Expected number, found {:?}", token)))
        }
    }

    fn binary(&mut self, f: fn (Box<HuckAst>, Box<HuckAst>) -> HuckAst, prec: Prec, lhs: HuckAst) -> ParseResult {
        let rhs = self.parse_prec(Prec::next(prec))?;
        Ok(f(Box::new(lhs), Box::new(rhs)))
    }

    fn plus(&mut self, _token: Token<'a>, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Plus, Prec::AddSub, lhs)
    }

    fn minus(&mut self, _token: Token<'a>, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Minus, Prec::AddSub, lhs)
    }

    fn times(&mut self, _token: Token<'a>, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Times, Prec::MultDiv, lhs)
    }

    fn div(&mut self, _token: Token<'a>, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Div, Prec::MultDiv, lhs)
    }

    fn grouping(&mut self, _token: Token<'a>) -> ParseResult {
        let grouping = self.parse_prec(Prec::Expr)?;
        self.consume(Token::RParen)?;
        Ok(grouping)
    }

    fn block(&mut self, _token: Token<'a>) -> ParseResult {
        let mut exprs = vec![];
        exprs.push(self.parse_prec(Prec::Expr)?);

        let mut next = self.tokens.peek();

        while next.is_some() && *next.unwrap() == Token::Semicolon {
            self.consume(Token::Semicolon)?;
            exprs.push(self.parse_prec(Prec::Expr)?);

            next = self.tokens.peek();
        }
        self.consume(Token::RBrace)?;
        Ok(HuckAst::Block(exprs))
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
        assert_eq!(parsed, Ok(HuckAst::Num(42)));
    }

    #[test]
    fn arithmetic() {
        let scanner = make_scanner("1 - 2 * 3");
        let parsed = Parser::new(scanner).parse();

        assert_eq!(parsed, Ok(
            HuckAst::Minus(
                Box::new(HuckAst::Num(1)),
                Box::new(HuckAst::Times(
                    Box::new(HuckAst::Num(2)),
                    Box::new(HuckAst::Num(3))
                ))
            )
        ));
    }

    #[test]
    fn grouping() {
        let scanner = make_scanner("(1 + 2) / 3");
        let parsed = Parser::new(scanner).parse();

        assert_eq!(parsed, Ok(
            HuckAst::Div(
                Box::new(HuckAst::Plus(
                    Box::new(HuckAst::Num(1)),
                    Box::new(HuckAst::Num(2))
                )),
                Box::new(HuckAst::Num(3))
            )
        ));
    }

    #[test]
    fn nested_grouping() {
        let scanner = make_scanner("(((420)))");
        assert_eq!(
            Parser::new(scanner).parse(),
            Ok(HuckAst::Num(420))
        )
    }

    #[test]
    fn bad_grouping() {
        let scanner = make_scanner("(2580");
        assert!(Parser::new(scanner).parse().is_err())
    }
}
