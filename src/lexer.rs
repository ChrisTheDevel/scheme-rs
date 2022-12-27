//std lib imports
use std::collections::HashSet;
use std::{iter::Peekable, str::Chars};
// external imports
use lazy_static::lazy_static;
// internal imports
use crate::tokens::{LiteralKind, Token};

lazy_static! {
    /// extended identification chars
    static ref EXTENDED_IDENT_CHARS: HashSet<char> = HashSet::from(['!', '$', '%', '&', '*', '+', '-', '.', '/', ':', '<', '=', '>', '?', '@', '^', '_', '~']);
}

/// Directives that designate whether an identifier should use be case agnostic
const DIRECTIVES: [&'static str; 2] = ["#!fold-case", "#!no-fold-case"];

pub const EOF_CHAR: char = '\0';

/// The Lexer. Taking heavy inspiration of the rustc_lexer Cursor struct
pub struct Lexer<'a> {
    len_remaining: usize,
    chars: Chars<'a>,
}

// Here we implement some tooling
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            len_remaining: input.len(),
            chars: input.chars(),
        }
    }

    // peeks the next char
    fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    // peeks the second char
    fn second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn take_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> String {
        let mut s = String::new();
        while predicate(self.first()) && !self.is_eof() {
            s.push(self.bump().unwrap());
        }
        s
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
}

// here we define the syntax specific tooling
impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Token {
        use Token::*;
        // try consuming a char
        let first_char = match self.bump() {
            Some(c) => c,
            None => return EOF,
        };
        let token_kind = match first_char {
            ';' => self.line_comment(),
            '#' => match self.first() {
                '!' => self.directive(),
                '|' => self.block_comment(),
                '(' => OpenVec,
                // ';' => TODO datum comments
                'u' => self.byte_vec(),
                // #e,i,b,o,d,x => notation in numbers numbers
                _ => Unknown,
            },
            '"' => self.string_literal()
            _ => Unknown,
        };

        // if we've been unsuccessfull in  matching some known syntax,
        Unknown
    }
    fn string_literal(&mut self) -> Token {
        // throw away the '"'
        self.bump();
        let content = self.take_while(|c| !c.is_whitespace());
        Token::Directive(content)
    }

    fn byte_vec(&mut self) -> Token {
        // throw away the 'u'
        self.bump();
        // check that the next 2 chars are '8' and '('
        let mut chars = self.chars.clone();
        if chars.next().unwrap() == '8' && chars.next().unwrap() == '(' {
            // if it indeed was a byte vec, then consume the chars
            self.bump();
            self.bump();
            Token::OpenByteVec
        } else {
            Token::Unknown
        }
    }

    fn line_comment(&mut self) -> Token {
        let content = self.take_while(|c| c != '\n');
        Token::Comment(content)
    }

    fn block_comment(&mut self) -> Token {
        // throw away the '|'
        self.bump();
        let mut content = String::new();
        while self.first() != '|' && self.second() != '#' && !self.is_eof() {
            content.push(self.bump().unwrap());
        }
        // throw away the '|' and '#'
        self.bump();
        self.bump();
        Token::BlockComment(content)
    }

    fn directive(&mut self) -> Token {
        // throw away the '!'
        self.bump();
        let content = self.take_while(|c| !c.is_whitespace());
        Token::Directive(content)
    }
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || EXTENDED_IDENT_CHARS.contains(&c)
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;

    /// Tests that the sequences of tokens produced by the lexer matches the expected sequence.
    fn expected_sequnce(seq: &[Token], input: &str) {
        let mut lexer = Lexer::new(input);
        for token in seq {
            assert_eq!(*token, lexer.next_token());
        }
        assert_eq!(EOF, lexer.next_token());
    }

    #[test]
    fn no_input() {
        expected_sequnce(&[], "");
    }

    #[test]
    fn parens() {
        expected_sequnce(
            &[
                OpenParen, OpenParen, OpenParen, OpenParen, CloseParen, CloseParen, CloseParen,
                CloseParen,
            ],
            "(((())))",
        );
    }
    #[test]
    fn comment1() {
        expected_sequnce(
            &[Comment(String::from("this is a comment"))],
            "; this is a comment",
        );
    }

    #[test]
    fn ident1() {
        expected_sequnce(
            &[
                OpenParen,
                Identifier(String::from("+")),
                Identifier(String::from("var1")),
                Identifier(String::from("var2")),
                CloseParen,
            ],
            "(+ var1 var2)",
        );
    }
    #[test]
    fn test_multiple_ients() {
        let idents = [
            "...",
            "+",
            "+soup+",
            "<=?",
            "->string",
            "a34kTMNs",
            "lambda",
            "list->vector",
            "q",
            "V17a",
            "|two words|",
            "|two\x20;words|",
            "the-word-recursion-has-many-meanings",
        ];
        for ident in idents {
            expected_sequnce(&[Identifier(String::from(ident))], ident);
        }
    }
}
