//std lib imports
use std::collections::HashSet;
use std::{iter::Peekable, str::Chars};
// external imports
use lazy_static::lazy_static;
// internal imports

lazy_static! {
    /// extended identification chars
    static ref EXTENDED_IDENT_CHARS: HashSet<char> = HashSet::from(['!', '$', '%', '&', '*', '+', '-', '.', '/', ':', '<', '=', '>', '?', '@', '^', '_', '~']);
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let input = input.chars().peekable();
        Self { input }
    }

    pub fn next_token(&mut self) -> Token {
        use Token::*;

        let read_identifier = |l: &mut Lexer| -> Vec<char> {
            let mut identifier = Vec::new();
            while let Some(c) = l.input.peek() {
                if is_identifier_char(*c) {
                    identifier.push(*c);
                    let _ = l.input.next();
                } else {
                    break;
                }
            }
            identifier
        };

        let c = self.input.next();
        if c.is_none() {
            return EOF;
        }

        let mut c = c.unwrap();
        // get the next non_whitespace character
        while c.is_whitespace() {
            match self.input.next() {
                Some(new_c) => c = new_c,
                None => return EOF,
            }
        }
        match c {
            '(' => LParen,
            '{' => LCParen,
            '[' => LSParen,
            ')' => RParen,
            '}' => RCParen,
            ']' => RSParen,
            ';' => {
                let mut comment = String::new();

                while let Some(c) = self.input.peek() {
                    if *c == '\n' {
                        break;
                    }
                    comment.push(self.input.next().unwrap());
                }
                // TODO can I skip this cloning somehow?
                Comment(comment.trim().into())
            }
            '"' => {
                let mut s = String::new();
                while let Some(c) = self.input.peek() {
                    if *c == '"' {
                        // throw away the '"'
                        let _ = self.input.next();
                        break;
                    }
                    s.push(self.input.next().unwrap());
                }
                StringLiteral(s)
            }
            '|' => {
                let mut identifier = String::from(c);
                while let Some(c) = self.input.peek() {
                    if *c == '|' {
                        identifier.push(self.input.next().unwrap());
                        break;
                    }
                    identifier.push(self.input.next().unwrap());
                }
                Identifier(identifier)
            }
            _ => {
                let mut identifier = String::from(c);
                while let Some(c) = self.input.peek() {
                    // TODO make rules for including identifier more specific
                    if !is_identifier_char(*c) {
                        break;
                    }
                    identifier.push(self.input.next().unwrap());
                }
                Identifier(identifier)
            }
        }
    }
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || EXTENDED_IDENT_CHARS.contains(&c)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Identifier(String), // any sequence of letters, numbers and extended symbos "! $ % & * + - . / : < = > ? @ ^ _ ~" (a single "." is not a valid token though).
    Comment(String),    // ;;comment to end of line, |# block comment #|
    Directive(String),  // #!directive
    // parenthesis
    LParen,  // (
    RParen,  // )
    LSParen, // Reserved
    RSParen, // Reserved
    LCParen, // Reserved
    RCParen, // Reserved
    // Literals
    StringLiteral(String),
    // Last token generated
    EOF, // end of file
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
                LParen, LParen, LParen, LParen, RParen, RParen, RParen, RParen,
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
                LParen,
                Identifier(String::from("+")),
                Identifier(String::from("var1")),
                Identifier(String::from("var2")),
                RParen,
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

    #[test]
    fn test_string() {
        expected_sequnce(
            &[StringLiteral(String::from("testing"))],
            r#"   "testing"    "#,
        );
    }
}
