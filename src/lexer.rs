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

    // peeks the third char
    fn third(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
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
        // Based on some char patterns we will opportunistically try to consume more of the input.
        // Every method used to consume further might however return `Token::Unknown` instead if they were
        // unable to parse the consumed chars as expected.
        let token_kind = match (first_char, self.first(), self.second(), self.third()) {
            // Single char tokens
            ('(', _, _, _) => OpenParen,
            (')', _, _, _) => CloseParen,
            ('[', _, _, _) => OpenSquareParen, // reserved for future syntax extensions
            (']', _, _, _) => CloseSquareParen, // reserved
            ('{', _, _, _) => OpenCurlyParen,  // reserved
            ('}', _, _, _) => CloseCurlyParen, // reserved
            ('\'', _, _, _) => Apost,          // denotes literal data
            ('`', _, _, _) => Grave,           // denotes partially constant data
            // comments
            (';', _, _, _) => self.line_comment(),
            ('#', '|', _, _) => self.block_comment(),
            // directive
            ('#', '!', _, _) => self.directive(),
            ('#', c, _, _) if c == 't' || c == 'f' => self.boolean(),
            // some list types
            ('#', 'u', '8', '(') => self.bytevector(),
            ('#', '(', _, _) => self.vector(),
            (w, _, _, _) if w.is_whitespace() => self.whitespace(),
            // identifiers
            ('|', _, _, _) => self.pipe_identifier(),
            (i, _, _, _) if is_valid_first_letter_ident(i) => self.identifier(i),
            _ => Unknown,
        };

        // if we've been unsuccessfull in  matching some known syntax,
        token_kind
    }

    fn identifier(&mut self, first_letter: char) -> Token {
        // we will now try to parse the first
        let mut content = String::from(first_letter);
        loop {
            let next = self.first();

            if next == '\n' {
                break;
            }
            if is_identifier_char(next) {
                // the char was a valid identificator char, continue on
                content.push(self.bump().unwrap());
            } else {
                // we've encontered a nonvalid char!
                // consume untill the next whitespace and return unknown
                self.eat_while(|c| !c.is_whitespace());
                return Token::Unknown;
            }
        }
        Token::Identifier(content)
    }
    fn vector(&mut self) -> Token {
        self.bump(); // throw away the '('
        Token::OpenVec
    }

    fn bytevector(&mut self) -> Token {
        self.bump(); // throw away the 'u'
        self.bump(); // throw away the '8'
        self.bump(); // throw away the '('
        Token::OpenByteVec
    }

    fn boolean(&mut self) -> Token {
        let content = self.take_while(|c| c.is_whitespace());
        let c = &content;
        if c == "t" || c == "true" || c == "f" || c == "false" {
            Token::Literal(LiteralKind::Boolean(content))
        } else {
            Token::Unknown
        }
    }

    fn pipe_identifier(&mut self) -> Token {
        // throw away the '|'
        self.bump();
        let content = self.take_while(|c| c == '|');
        let res = Token::Identifier(content);
        self.bump();
        res
    }

    fn whitespace(&mut self) -> Token {
        self.eat_while(|c| c.is_whitespace());
        Token::Whitespace
    }

    fn string_literal(&mut self) -> Token {
        // throw away the '"'
        self.bump();
        let content = self.take_while(|c| !c.is_whitespace());
        let res = Token::Literal(LiteralKind::Str(content));
        // trow away the second '"'
        self.bump();
        res
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

/// checks whether the letter i is a valid first letter of an identifier
/// (can't be a number or invalid extended char)
fn is_valid_first_letter_ident(c: char) -> bool {
    !c.is_numeric() && (c.is_alphabetic() || EXTENDED_IDENT_CHARS.contains(&c))
}

/// checks whether a letter is a valid ident letter.
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
