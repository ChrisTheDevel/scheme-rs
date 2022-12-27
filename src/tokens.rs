#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    /// any whitespace
    Whitespace,
    /// a-z,A-Z,1-9,extended symbos "! $ % & * + - . / : < = > ? @ ^ _ ~" (a single "." is not a valid token though).
    /// cannot also start with number
    Identifier(String),
    Comment(String),      // ;;comment to end of line
    BlockComment(String), // |# block comment #|
    Directive(String),    // #!directive
    // parenthesis
    OpenParen,        // (
    CloseParen,       // )
    OpenSquareParen,  // Reserved
    CloseSquareParen, // Reserved
    OpenCurlyParen,   // Reserved
    CloseCurlyParen,  // Reserved
    Apost,            // the ' char.
    // Open paren for some list types
    OpenVec,     // #(
    OpenByteVec, // #u8(
    // Literals
    Literal {
        literal_kind: LiteralKind,
    },
    /// Unknown token. Input contains non-defined syntax, or that couldn't be parsed!
    Unknown,
    // Last token generated. Every token stream should end with it.
    EOF, // end of file
}

#[derive(Debug, PartialEq, Eq)]
pub enum LiteralKind {
    Str(String),
    Boolean(String),
    Number(String),
}
