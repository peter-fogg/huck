use std::vec::IntoIter;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType { // Unify TokenType and Token???
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
    source: Peekable<IntoIter<(usize, char)>>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner{
            source: source.char_indices().collect::<Vec<_>>().into_iter().peekable(),
        }
    }

    fn number(&mut self, c: char) -> Option<Token> {
        let mut number_string: String = c.to_string();
        while is_digit(self.source.peek()?.1) {
            number_string.push(self.source.next()?.1);
        }
        return Some(Token{token_type: Number, chars: number_string});
    }

}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.source.next()?.1;
            match c {
                c if is_whitespace(c) => { continue; }
                c if is_digit(c) => {
                    return self.number(c)
                },
                '+' => return Some(make_token(Plus, "+")),
                '-' => return Some(make_token(Minus, "-")),
                '*' => return Some(make_token(Star, "*")),
                '/' => return Some(make_token(Slash, "/")),
                '(' => return Some(make_token(LParen, "(")),
                ')' => return Some(make_token(RParen, ")")),
                _ => return None,
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
