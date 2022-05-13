use crate::scanner::{TokenType, Token};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq)]
pub enum HuckAst {
    Num(u64),
    Plus(Box<HuckAst>, Box<HuckAst>),
    Minus(Box<HuckAst>, Box<HuckAst>),
    Times(Box<HuckAst>, Box<HuckAst>),
    Div(Box<HuckAst>, Box<HuckAst>),
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

type TokenStream = Peekable<IntoIter<Token>>;

type ParseResult = Result<HuckAst, ParseError>;

type PrefixRule = fn(&mut Parser, token: Token) -> ParseResult;

type InfixRule = fn(&mut Parser, token: Token, lhs: HuckAst) -> ParseResult;

pub struct Parser {
    tokens: TokenStream,
}

impl Parser {
    pub fn new(tokens: TokenStream) -> Self {
        Self { tokens: tokens }
    }

    pub fn parse(&mut self) -> ParseResult {
        self.parse_prec(Prec::Bottom)
    }

    fn parse_prec(&mut self, prec: Prec) -> ParseResult {
        let t = self.tokens.next().ok_or(ParseError::Eof)?;

        let prefix_rule = Self::get_prefix_rule(t.token_type)?;
        let mut lhs = prefix_rule(self, t)?;

        let mut next_prec = self.tokens.peek().map(|next| Self::get_prec(next.token_type));
        while next_prec.is_some() && prec <= next_prec.unwrap() {

            let next = self.tokens.next().unwrap();

            let t_type = next.token_type;

            let infix_rule = Self::get_infix_rule(t_type)?;

            lhs = infix_rule(self, next, lhs)?;
            next_prec = self.tokens.peek().map(|next| Self::get_prec(next.token_type));
        }

        Ok(lhs)
    }

    fn number(&mut self, token: Token) -> ParseResult {
        if let Ok(num) = token.chars.parse() {
            Ok(HuckAst::Num(num))
        } else {
            Err(ParseError::Fucked(format!("Failed to parse number {}", token.chars)))
        }
    }

    fn binary(&mut self, f: fn (Box<HuckAst>, Box<HuckAst>) -> HuckAst, prec: Prec, lhs: HuckAst) -> ParseResult {
        let rhs = self.parse_prec(Prec::next(prec))?;
        Ok(f(Box::new(lhs), Box::new(rhs)))
    }

    fn plus(&mut self, _token: Token, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Plus, Prec::AddSub, lhs)
    }

    fn minus(&mut self, _token: Token, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Minus, Prec::AddSub, lhs)
    }

    fn times(&mut self, _token: Token, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Times, Prec::MultDiv, lhs)
    }

    fn div(&mut self, _token: Token, lhs: HuckAst) -> ParseResult {
        self.binary(HuckAst::Div, Prec::MultDiv, lhs)
    }

    fn grouping(&mut self, _token: Token) -> ParseResult {
        let grouping = self.parse_prec(Prec::Expr)?;
        self.consume(TokenType::RParen)?;
        Ok(grouping)
    }

    fn consume(&mut self, token_type: TokenType) -> Result<(), ParseError> {
        match self.tokens.peek() {
            Some(c) if c.token_type == token_type => {
                _ = self.tokens.next();
                Ok(())
            },
            _ => Err(ParseError::Fucked(format!("Expected token type {:?}", token_type))),
        }
    }

    fn get_infix_rule(t: TokenType) -> Result<InfixRule, ParseError> {
        match t {
            TokenType::Plus => Ok(Self::plus),
            TokenType::Minus => Ok(Self::minus),
            TokenType::Star => Ok(Self::times),
            TokenType::Slash => Ok(Self::div),
            _ => Err(ParseError::NotImplemented(format!("No infix rule for token type {:?}", t))),
        }
    }
    
    fn get_prefix_rule(t: TokenType) -> Result<PrefixRule, ParseError> {
        match t {
            TokenType::Number => Ok(Self::number),
            TokenType::LParen => Ok(Self::grouping),
            _ => Err(ParseError::NotImplemented(format!("No prefix rule for token type {:?}", t))),
        }
    }

    fn get_prec(t: TokenType) -> Prec {
        match t {
            TokenType::Number => Prec::Expr,
            TokenType::Plus => Prec::AddSub,
            TokenType::Minus => Prec::AddSub,
            TokenType::Star => Prec::MultDiv,
            TokenType::Slash => Prec::MultDiv,
            _ => Prec::Bottom,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::scanner::{make_token, TokenType};

    #[test]
    fn empty() {
        let tokens = vec![].into_iter().peekable();
        let parsed = Parser::new(tokens).parse();
        assert_eq!(parsed, Err(ParseError::Eof));
    }

    #[test]
    fn number() {
        let tokens = vec![make_token(TokenType::Number, "42")].into_iter().peekable();
        let parsed = Parser::new(tokens).parse();
        assert_eq!(parsed, Ok(HuckAst::Num(42)));
    }

    #[test]
    fn arithmetic() {
        let tokens = vec![
            make_token(TokenType::Number, "1"),
            make_token(TokenType::Minus, "-"),
            make_token(TokenType::Number, "2"),
            make_token(TokenType::Star, "*"),
            make_token(TokenType::Number, "3"),
        ].into_iter().peekable();
        let parsed = Parser::new(tokens).parse();

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
        let tokens = vec![
            make_token(TokenType::LParen, "("),
            make_token(TokenType::Number, "1"),
            make_token(TokenType::Plus, "+"),
            make_token(TokenType::Number, "2"),
            make_token(TokenType::RParen, ")"),
            make_token(TokenType::Slash, "/"),
            make_token(TokenType::Number, "3"),
        ].into_iter().peekable();
        let parsed = Parser::new(tokens).parse();

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
        let tokens = vec![
            make_token(TokenType::LParen, "("),
            make_token(TokenType::LParen, "("),
            make_token(TokenType::LParen, "("),
            make_token(TokenType::Number, "420"),
            make_token(TokenType::RParen, ")"),
            make_token(TokenType::RParen, ")"),
            make_token(TokenType::RParen, ")"),
        ].into_iter().peekable();

        assert_eq!(
            Parser::new(tokens).parse(),
            Ok(HuckAst::Num(420))
        )
    }

    #[test]
    fn bad_grouping() {
        let tokens = vec![
            make_token(TokenType::LParen, "("),
            make_token(TokenType::Number, "2580"),
        ].into_iter().peekable();

        assert!(Parser::new(tokens).parse().is_err())
    }
}
