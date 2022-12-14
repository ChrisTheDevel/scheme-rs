#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    /// a-z,A-Z,1-9,extended symbos "! $ % & * + - . / : < = > ? @ ^ _ ~" (a single "." is not a valid token though).
    /// cannot also start with number
    Identifier(String),
    /// Identifier enclosed with '|', has some special rules in it's contents
    PipeIdentifier(String),
    Comment(String),      // ;;comment to end of line
    BlockComment(String), // |# block comment #|
    Directive(String),    // #!directive
    DatumOpen(String),    // #number= - e.g #323=
    DatumRef(String),     // #number#
    // parenthesis
    OpenParen,        // (
    CloseParen,       // )
    OpenSquareParen,  // Reserved
    CloseSquareParen, // Reserved
    OpenCurlyParen,   // Reserved
    CloseCurlyParen,  // Reserved
    Apost,            // the ' char. Denotes literal data.
    Grave,            // the ` char. Denotes partially constant data.
    // Open paren for some list types
    OpenVec,     // #(
    OpenByteVec, // #u8(
    // Literals
    Literal(LiteralKind),
    /// Unknown token. Input contains non-defined syntax, or that couldn't be parsed!
    Error,
    // Last token generated. Every token stream should end with it.
    EOF, // end of file
}

#[derive(Debug, PartialEq, Eq)]
pub enum LiteralKind {
    Str(String),
    Boolean(String),
    Number(String),
}
