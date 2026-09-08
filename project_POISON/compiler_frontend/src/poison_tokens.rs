use crate::poison_semantic_analyzer::poison_symbol_table::IdentifierId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokensKind {
    CharLiteral,
    StringLiteral,
    IntegerLiteral,
    FloatLiteral,
    BoolLiteral,

    Identifier,

    KeywordUint64,
    KeywordUint32,
    KeywordUint16,
    KeywordUint8,

    KeywordFloat64,
    KeywordFloat32,
    KeywordFloat16,
    KeywordFloat8,

    KeywordInt64,
    KeywordInt32,
    KeywordInt16,
    KeywordInt8,

    KeywordMutable, // init mutable x: Int32 = 40;

    KeywordWhile,
    KeywordFor,
    KeywordStructure,

    KeywordIf,
    KeywordElse,
    KeywordConnect,
    KeywordFunc,
    KeywordReturn,

    KeywordInit,
    KeywordDec,

    KeywordChar,
    KeywordString,

    KeywordTrue,
    KeywordFalse,

    Asterisk,
    Plus,
    Minus,
    Slash,

    Assign,
    EqualEqual,
    BangEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    AndAnd,
    PipePipe,
    Bang,
    ColonArrow,
    HashTag,
    Attribute,
    
    KeywordBool,
    KeywordIn,
    KeywordVolatile,
    KeywordDotDot,
    KeywordDotDotEqual,
    KeywordStep,
    TypeRange,

    AddBy, // x += 3;
    SubtractBy, // x -= 3;
    MultiplyBy, // x *= 3;
    DivideBy, // x /= 3;

    DoubleQuote,
    SingleQuote,

    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Lbracket,
    Rbracket,
    AmperSand,
    At,

    Colon,
    Semicolon,
    
    Eof,

    Comma,
    Dot,
}
#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokensKind,
    pub span: Span,
    pub id: Option<IdentifierId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}