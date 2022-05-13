use std::vec::IntoIter;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Number,
    RParen,
    LParen
}
use TokenType::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub chars: String
}

#[derive(Debug)]
pub struct Scanner {
    source: Peekable<IntoIter<char>>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner{
            source: source.chars().collect::<Vec<_>>().into_iter().peekable(),
        }
    }

    fn number(&mut self, c: char) -> Option<Token> {
        let mut number_string: String = c.to_string();
        while is_digit(*self.source.peek()?) {
            number_string.push(self.source.next()?);
        }
        return Some(Token{token_type: Number, chars: number_string});
    }

}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.source.next();
            match c {
                Some(c) if is_whitespace(c) => { continue; }
                Some(c) if is_digit(c) => {
                    return self.number(c)
                },
                Some('+') => {
                    return Some(make_token(Plus, "+"))
                },
                Some('-') => {
                    return Some(make_token(Minus, "-"))
                },
                Some('*') => {
                    return Some(make_token(Star, "*"))
                },
                Some('/') => {
                    return Some(make_token(Slash, "/"))
                },
                Some('(') => {
                    return Some(make_token(LParen, "("))
                },
                Some(')') => {
                    return Some(make_token(RParen, ")"))
                },
                _ => {
                    return None;
                }
            };
        }
    }
}

fn is_digit(c: char) -> bool {
    "0123456789".contains(c)
}

fn is_whitespace(c: char) -> bool {
    " \t\n".contains(c)
}

pub fn make_token(token_type: TokenType, chars: &str) -> Token {
    Token{token_type: token_type, chars: chars.to_string()}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn whitespace() {
        let tokens = Scanner::new(" \t      \n\n  \n").collect::<Vec<_>>();

        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn numbers() {
        let tokens = Scanner::new("1124\n").collect::<Vec<_>>();

        assert_eq!(tokens, vec![make_token(Number, "1124")]);
    }

    #[test]
    fn operators() {
        let tokens = Scanner::new("* - + /\n").collect::<Vec<_>>();

        assert_eq!(tokens, vec![
            make_token(Star, "*"),
            make_token(Minus, "-"),
            make_token(Plus, "+"),
            make_token(Slash, "/"),
        ]);
    }
}
