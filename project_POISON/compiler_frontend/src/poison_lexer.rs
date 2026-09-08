use crate::{poison_semantic_analyzer::poison_symbol_table::Interner, poison_source_stream::SourceStream, poison_tokens::*};

pub struct Lexer<'a> {
    pub stream: SourceStream<'a>,
    pub interner: &'a mut Interner,
}
impl<'a> Lexer<'a> {
    #[inline(always)]
    pub fn new(source: &'a str, interner: &'a mut Interner) -> Self {
        Self {
            stream: SourceStream::new(source.as_bytes()),
            interner,
        }
    }
    #[inline(always)]
    pub fn get_text(&self, span: Span) -> &'a str {
        unsafe {
            let bytes = self.stream.get_slice(span.start as usize, span.end as usize);
            std::str::from_utf8_unchecked(bytes)
        }
    }
    #[inline(always)]
    pub fn next_token(&mut self) -> Token {
        self.stream.skip_whitespace();
        let start = self.stream.index();
        let current_byte = match self.stream.advance() {
            Some(b) => b,
            None => return Token { kind: TokensKind::Eof, span: Span { start, end: start },id: None },
        };

        match current_byte {
            b'*' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::MultiplyBy, span: Span { start, end: start + 2 },id: None};
                }
                Token { kind: TokensKind::Asterisk, span: Span { start, end: start + 1 },id: None}
            },
            b'+' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::AddBy, span: Span { start, end: start + 2 },id: None};
                }
                Token { kind: TokensKind::Plus, span: Span { start, end: start + 1 },id: None}
            },
            b'-' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::SubtractBy, span: Span { start, end: start + 2 },id: None};
                }
                Token { kind: TokensKind::Minus, span: Span { start, end: start + 1 },id: None}
            },
            b'/' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::DivideBy, span: Span { start, end: start + 2 },id: None};
                }
                Token { kind: TokensKind::Slash, span: Span { start, end: start + 1 },id: None}
            },
            b'@' => Token { kind: TokensKind::At, span: Span { start, end: start + 1 },id: None},
            b':' => {
                if self.stream.peek() == Some(b'>') {
                    self.stream.advance();
                    return Token { kind: TokensKind::ColonArrow, span: Span { start, end: start + 2 },id: None};
                }
                Token { kind: TokensKind::Colon, span: Span { start, end: start + 1 },id: None}
            },
            b'#' => {
                //i have to handle #connect in some way
                // keywordConnect
                Token { kind: TokensKind::HashTag, span: Span { start, end: start + 1 },id: None}
            },
            b',' => Token { kind: TokensKind::Comma, span: Span { start, end: start + 1 },id: None},
            b';' => Token { kind: TokensKind::Semicolon, span: Span { start, end: start + 1 },id: None},

            b'[' => Token { kind: TokensKind::Lbracket, span: Span { start, end: start + 1 },id: None},
            b']' => Token { kind: TokensKind::Rbracket, span: Span { start, end: start + 1 },id: None},
            b'{' => Token { kind: TokensKind::Lbrace, span: Span { start, end: start + 1 },id: None},
            b'}' => Token { kind: TokensKind::Rbrace, span: Span { start, end: start + 1 },id: None},
            b'(' => Token { kind: TokensKind::Lparen, span: Span { start, end: start + 1 },id: None},
            b')' => Token { kind: TokensKind::Rparen, span: Span { start, end: start + 1 },id: None},

            b'"' => self.lex_string_literal(start),

            b'=' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::EqualEqual, span: Span { start, end: start + 2 },id: None}
                }
                Token { kind: TokensKind::Assign, span: Span { start, end: start + 1 },id: None}
            },
            b'!' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::BangEqual, span: Span { start, end: start + 2 },id: None}
                }
                Token { kind: TokensKind::Bang, span: Span { start, end: start + 1 },id: None}
            },
            b'.' => {
                if self.stream.peek() == Some(b'.') {
                    self.stream.advance();
                    if self.stream.peek() == Some(b'=') {
                        self.stream.advance();
                        return Token { kind: TokensKind::KeywordDotDotEqual, span: Span { start, end: start + 3 },id: None}
                    }
                    return Token { kind: TokensKind::KeywordDotDot, span: Span { start, end: start + 2 },id: None}
                }
                Token { kind: TokensKind::Dot, span: Span { start, end: start + 1 },id: None}
            },
            b'>' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::GreaterThanEqual, span: Span { start, end: start + 2 },id: None}
                }
                Token { kind: TokensKind::GreaterThan, span: Span { start, end: start + 1 },id: None}
            },
            b'&' => {
                if self.stream.peek() == Some(b'&') {
                    self.stream.advance();
                    return Token { kind: TokensKind::AndAnd, span: Span { start, end: start + 2 },id: None}
                }
                Token { kind: TokensKind::AmperSand, span: Span { start, end: start + 1 },id: None}
            },

            b'<' => {
                if self.stream.peek() == Some(b'=') {
                    self.stream.advance();
                    return Token { kind: TokensKind::LessThanEqual, span: Span { start, end: start + 2 },id: None}
                }
                Token { kind: TokensKind::LessThan, span: Span { start, end: start + 1 },id: None}
            },

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_identifier_or_keyword(start), // for later,
            b'0'..=b'9' => self.lex_number(start,current_byte),
            _ => panic!("Lexer Error: Unexpected byte code '{}'", current_byte as char),
      }
    }
    #[inline(always)]
    fn lex_string_literal(&mut self, start: usize) -> Token {
        let base = self.stream.initial_ptr;
        let len = self.stream.length;
        let mut idx = self.stream.index;

        let mut escaped = false;
        
        while idx < len {
            let b = unsafe { *base.add(idx) };
            
            if escaped {
                escaped = false;
                idx += 1;
                continue;
            }
            
            match b {
                b'\\' => {
                    escaped = true;
                    idx += 1;
                }
                b'"' => {
                    idx += 1;
                    self.stream.index = idx;

    
                    let raw_bytes = unsafe {
                        std::slice::from_raw_parts(base.add(start + 1), (idx - 1) - (start + 1))
                    };
                    let clean_text = unsafe { std::str::from_utf8_unchecked(raw_bytes) };

                    let string_id = self.interner.intern(clean_text);

                    return Token {
                        kind: TokensKind::StringLiteral,
                        span: Span { start, end: idx },
                        id: Some(string_id)
                    };
                }
                _ => {
                    idx += 1;
                }
            }
        }

        panic!("Lexer Error: Unterminated string literal starting at position {}", start);
    }
    #[inline(always)]
    fn lex_number(&mut self,start: usize,first_byte: u8) -> Token{
        let mut is_float = false;

        if first_byte == b'0' {
            match self.stream.peek() {
                Some(b'x' | b'X') => {
                    self.stream.advance();
                    return self.lex_hex_literal(start);
                },
                Some(b'b' | b'B') => {
                    self.stream.advance();
                    return self.lex_bin_literal(start);
                },
                _ => {}
            }
        }
        while let Some(b) = self.stream.peek() {
            match b { 
                b'0'..=b'9' => {
                    self.stream.advance();
                },
                b'_' => {
                    self.stream.advance();
                },
                _ => break,
            }
        }

        if self.stream.peek() == Some(b'.') {
            if let Some(next_byte) = self.stream.peek_at(1) {
                if next_byte.is_ascii_digit() {
                    is_float = true;
                    self.stream.advance();
                    while let Some(b) = self.stream.peek() {
                        match b {
                            b'0'..=b'9' | b'_' => {
                                 self.stream.advance(); 
                            }
                            _ => break,
                        }
                    }
                }
            }
        }
        let kind = if is_float {TokensKind::FloatLiteral} else {TokensKind::IntegerLiteral};
        Token {kind, span: Span {start, end: self.stream.index()}, id: None}
    }
    #[inline(always)]
    fn lex_hex_literal(&mut self, start: usize) -> Token{
        while let Some(b) = self.stream.peek() {
            match b {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' | b'_' => {
                    self.stream.advance();
                },
                _ => break,
            }
        }
        Token { kind: TokensKind::IntegerLiteral, span: Span { start, end: self.stream.index() },id: None }
    }
    #[inline(always)]
    fn lex_bin_literal(&mut self, start: usize) -> Token{
        while let Some(b) = self.stream.peek() {
            match b {
                b'0' | b'1' | b'_' => {
                    self.stream.advance();
                },
                _ => break,
            }
        }
        Token { kind: TokensKind::IntegerLiteral, span: Span { start, end: self.stream.index()}, id: None }
    } 
    pub fn lex_identifier_or_keyword(&mut self,start: usize) -> Token {
        let base = self.stream.initial_ptr;
        let len = self.stream.length;
        let mut idx = self.stream.index();

        while idx < len {
            let b = unsafe { *base.add(idx) };
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => {
                    idx += 1;
                }
                _ => break,
            }
        }
        self.stream.index = idx;

        let raw_bytes = unsafe {
            std::slice::from_raw_parts(base.add(start), idx - start)
        };
        let mut id = None;

        let kind = match raw_bytes {
            b"if" => TokensKind::KeywordIf,
            b"else" => TokensKind::KeywordElse,
            b"while" => TokensKind::KeywordWhile,
            b"for" => TokensKind::KeywordFor,
            b"connect" => TokensKind::KeywordConnect,
            b"mut" => TokensKind::KeywordMutable,
            b"step" => TokensKind::KeywordStep,
            b"volatile" => TokensKind::KeywordVolatile,
            b"Bool" => TokensKind::KeywordBool,
            b"in" => TokensKind::KeywordIn,

            b"true" => TokensKind::KeywordTrue,
            b"false" => TokensKind::KeywordFalse,

            b"init" => TokensKind::KeywordInit,
            b"dec" => TokensKind::KeywordDec,
            b"returns" => TokensKind::KeywordReturn,
            b"func" => TokensKind::KeywordFunc,
            b"structure" => TokensKind::KeywordStructure,

            b"Int64" => TokensKind::KeywordInt64,
            b"Int32" => TokensKind::KeywordInt32,
            b"Int16" => TokensKind::KeywordInt16,
            b"Int8" => TokensKind::KeywordInt8,
            
            b"Uint64" => TokensKind::KeywordUint64,
            b"Uint32" => TokensKind::KeywordUint32,
            b"Uint16" => TokensKind::KeywordUint16,
            b"Uint8" => TokensKind::KeywordUint8,

            b"Float64" => TokensKind::KeywordFloat64,
            b"Float32" => TokensKind::KeywordFloat32,
            b"Float16" => TokensKind::KeywordFloat16,
            b"Float8" => TokensKind::KeywordFloat8,
            b"String" => TokensKind::KeywordString,
            _ => {
                let lexeme = unsafe { std::str::from_utf8_unchecked(raw_bytes) };
                id = Some(self.interner.intern(lexeme));
                TokensKind::Identifier
            },
        };
        Token {
            kind, 
            span: Span {
                start,
                end: idx
            },
            id,
        }
    }
}