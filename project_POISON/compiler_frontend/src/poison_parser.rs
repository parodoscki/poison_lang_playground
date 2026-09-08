use crate::{poison_abstract_syntax_tree_nodes::*, poison_lexer::*,poison_tokens::*};
#[allow(unused)]


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    ExpectedToken { expected: TokensKind, found: TokensKind },
    ExpectedExpression,
    ExpectedTypeSignature,
    InvalidGlobalStatement,
    InvalidLiteral,
    UnterminatedBlock,
    InvalidAssignmentOperator,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub span: Span,
}

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub current_token: Token,
    pub peek_token: Token,
    pub tokens_trace: Vec<Token>, 
    pub errors: Vec<ParseError>,
}
impl<'a> Parser<'a> {
    #[inline(always)]
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self{
            lexer,
            current_token,
            peek_token,
            errors: Vec::new(),
            tokens_trace: Vec::with_capacity(1024), 
        }
    }
    #[inline(always)]
    pub fn report_errors(&self) {
        for err in &self.errors {
            match err.kind {
                ErrorKind::ExpectedToken { expected, found } => {
                    println!(
                        "Poison.SyntaxError: Expected token '{:?}' but found '{:?}' at position {}",
                        expected, found, err.span.start
                    );
                }
                ErrorKind::ExpectedTypeSignature => {
                    println!("Poison.SyntaxError: Missing or invalid type definition at position {}", err.span.start);
                }
                // Map the remaining variants smoothly...
                _ => println!("Poison.SyntaxError at position {}", err.span.start),
            }
        }
    }
    #[inline(always)]
    fn advance(&mut self) {
        self.tokens_trace.push(self.current_token);
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }
    #[inline(always)]
    fn parse_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError>{
        match self.current_token.kind {
            TokensKind::KeywordInit | TokensKind::KeywordDec => self.parse_var_or_dec(ast),
            TokensKind::KeywordWhile => self.parse_while_statement(ast),
            TokensKind::KeywordIf => self.parse_if_statement(ast),
            TokensKind::KeywordStructure => self.parse_structure_statement(ast),
            TokensKind::KeywordReturn => self.parse_return_statement(ast),
            TokensKind::HashTag => self.parse_connect_statement(ast),
            TokensKind::KeywordFunc => self.parse_func_statement(ast),
            TokensKind::KeywordFor => self.parse_for_loop_statement(ast),
            TokensKind::Lbrace => self.parse_block_statement(ast),
            _ => self.parse_expression_or_assignment(ast),
        }
    }
    #[inline(always)]
    pub fn parse_program(&mut self, ast: &mut Ast) -> Vec<StmtId> {
        let mut root_statements = Vec::new();
        while self.current_token.kind != TokensKind::Eof {
            match self.parse_global_declaration(ast) {
                Ok(stmt_id) => {
                    root_statements.push(stmt_id);
                }
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }
        root_statements
    }

    #[inline(always)]
    fn parse_global_declaration(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        match self.current_token.kind {
            TokensKind::KeywordFunc => self.parse_func_statement(ast),
            TokensKind::KeywordStructure => self.parse_structure_statement(ast),
            TokensKind::HashTag => self.parse_connect_statement(ast),
            
            TokensKind::KeywordReturn | TokensKind::KeywordWhile | 
            TokensKind::KeywordFor    | TokensKind::KeywordIf => {
                Err(ParseError {
                    kind: ErrorKind::InvalidGlobalStatement,
                    span: self.current_token.span,
                })
            }
            
            _ => {
                Err(ParseError {
                    kind: ErrorKind::InvalidGlobalStatement,
                    span: self.current_token.span,
                })
            }
        }
    }

    #[inline(always)]
    fn error<T>(&self, kind: ErrorKind) -> Result<T, ParseError> {
        Err(ParseError {
            kind,
            span: self.current_token.span,
        })
    }
    fn parse_block_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        self.advance(); // advance over '{'

        let mut statements = Vec::new();
        while self.current_token.kind != TokensKind::Rbrace &&
            self.current_token.kind != TokensKind::Eof {
                match self.parse_statement(ast) {
                    Ok(id) => {
                        statements.push(id);
                    },
                    Err(parse_error) => {
                        self.errors.push(parse_error);
                        self.synchronize();
                    },
                }
        }

        let end_span = self.current_token.span;
        if self.current_token.kind != TokensKind::Rbrace {
            return Err(
                ParseError {
                    kind: ErrorKind::UnterminatedBlock,
                    span: Span { start: start_span.start, end: end_span.end },
                }
            );
        }
        let stmtid = ast.add_statement(Statement::Block { 
            statements, 
            span: Span {
                start: start_span.start,
                end: end_span.end,
            }
        });
        Ok(stmtid)
    }
    fn parse_for_loop_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        self.advance(); // advance over 'for'

        if self.current_token.kind != TokensKind::Lbracket {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbracket, 
                found: self.current_token.kind 
            });
        }
        self.advance(); // advance over '['

        if self.current_token.kind != TokensKind::Identifier {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Identifier, 
                found: self.current_token.kind 
            });
        }

        let variable = self.current_token.span;
        self.advance(); // advance over identifier, example: for [(identifier) in 0..=40] {}

        if self.current_token.kind != TokensKind::KeywordIn {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::KeywordIn, 
                found: self.current_token.kind 
            });
        }
        self.advance(); // advance over 'in' keyword, example: for [i (in) 0..40]

        let range_start = self.parse_expression(ast, Precedence::Lowest)?;

        let is_inclusive = match self.current_token.kind {
            TokensKind::KeywordDotDot => {
                self.advance();
                false
            }
            TokensKind::KeywordDotDotEqual => {
                self.advance();
                true
            }
            _ => {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::TypeRange, 
                    found: self.current_token.kind 
                });
            }
        };

        let range_end = self.parse_expression(ast, Precedence::Lowest)?;
        let end_anchor = self.get_expr_span(ast, range_end);

        let step_by = if self.current_token.kind == TokensKind::Comma {
            self.advance(); // step over ,
            
            if self.current_token.kind != TokensKind::KeywordStep {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::KeywordStep, 
                    found: self.current_token.kind 
                });
            }
            self.advance(); // step over 'step' keyword

            self.parse_expression(ast, Precedence::Lowest)?
        } else {
            ast.add_expression(Expression::Literal {
                kind: LiteralKind::Integer(1),
                span: Span { start: end_anchor.end, end: end_anchor.end },
            })
        };
        if self.current_token.kind != TokensKind::Rbracket {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbracket, 
                found: self.current_token.kind 
            });
        }
        self.advance(); // step over ']'

        if self.current_token.kind != TokensKind::Lbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbrace, 
                found: self.current_token.kind 
            });
        }
        self.advance(); // step over '{'

        let mut body = Vec::new();
        while self.current_token.kind != TokensKind::Rbrace && self.current_token.kind != TokensKind::Eof {
            let stmt_id = self.parse_statement(ast)?;
            body.push(stmt_id);
        }

        if self.current_token.kind != TokensKind::Rbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbrace, 
                found: self.current_token.kind 
            });
        }
        let end_span = self.current_token.span;
        self.advance(); // step over '}'

        let stmt_id = ast.add_statement(Statement::For {
            variable,
            range_start,
            range_end,
            is_inclusive,
            step_by,
            body,
            span: Span {
                start: start_span.start,
                end: end_span.end,
            },
        });
        Ok(stmt_id)
    }




    fn synchronize(&mut self) {
        self.advance(); // skip past the token that caused the current syntax error
        
        while self.current_token.kind != TokensKind::Eof {
            if self.current_token.kind == TokensKind::Semicolon {
                self.advance();
                return;
            }
        
            match self.current_token.kind {
                TokensKind::KeywordFunc | TokensKind::KeywordStructure |
                TokensKind::KeywordIf   | TokensKind::KeywordWhile |
                TokensKind::KeywordInit | TokensKind::KeywordDec |
                TokensKind::KeywordFor  | TokensKind::KeywordReturn => return,
                _ => self.advance(), // Discard everything else
            }
        }
    }
    // parse_func would not be called directly, only in parse program


    fn parse_func_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        self.advance(); // steps over keyword func

        // supported attributes: @main @gpu @bare_metal
        let mut function_attribute = None;
        let name_span;
        let mut parameters = Vec::new();

        if self.current_token.kind == TokensKind::At {
            self.advance(); // steps over @

            if self.current_token.kind != TokensKind::Identifier {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Identifier, 
                    found: self.current_token.kind 
                });
            }
            let span = self.current_token.span;

            let attr_bytes = unsafe { 
                self.lexer.stream.get_slice(span.start as usize, span.end as usize) 
            };
            match attr_bytes.len() {
                4 if attr_bytes == b"main" => {
                    function_attribute = Some(span);
                }
                10 if attr_bytes == b"bare_metal" => {
                    function_attribute = Some(span);
                }
                3 if attr_bytes == b"gpu" => {
                    function_attribute = Some(span);
                }
                _ => {
                    return self.error(ErrorKind::ExpectedToken { 
                        expected: TokensKind::Attribute, 
                        found: self.current_token.kind 
                    });
                }
            }
            self.advance(); // step over attribute identifier
        }

        if self.current_token.kind == TokensKind::Identifier {
            name_span = self.current_token.span;
            self.advance();
        } else if self.current_token.kind == TokensKind::Lparen {
            if let Some(attr_span) = function_attribute {
                if self.lexer.get_text(attr_span) == "main" {
                    name_span = attr_span;
                } else {
                    return self.error(ErrorKind::ExpectedToken { 
                        expected: TokensKind::Identifier, 
                        found: self.current_token.kind 
                    });
                }
            } else {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Identifier, 
                    found: self.current_token.kind 
                });
            }
        } else {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Identifier, 
                    found: self.current_token.kind 
                });
        }

        if self.current_token.kind != TokensKind::Lparen {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lparen, 
                found: self.current_token.kind 
            });
        }
        self.advance(); // step over (

        if self.current_token.kind != TokensKind::Rparen {
            loop {
                if self.current_token.kind != TokensKind::Identifier {
                    return self.error(ErrorKind::ExpectedToken { 
                        expected: TokensKind::Identifier, 
                        found: self.current_token.kind 
                    });
                }
                let param_name_span = self.current_token.span;
                self.advance();

                if self.current_token.kind != TokensKind::Colon {
                    return self.error(ErrorKind::ExpectedToken { 
                        expected: TokensKind::Colon, 
                        found: self.current_token.kind
                    });
                }
                self.advance(); //step over ':'

                if !self.is_type_keyword(self.current_token.kind) {
                    return self.error(ErrorKind::ExpectedTypeSignature);
                }
                parameters.push(
                    Param {
                       name: param_name_span,
                       param_type: self.current_token.span 
                    }
                );
                self.advance();

                if self.current_token.kind == TokensKind::Comma {
                    self.advance(); // jump over ','
                } else {
                    break;
                }
            }
        }

        if self.current_token.kind != TokensKind::Rparen {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rparen, 
                found: self.current_token.kind
            });
        }
        self.advance(); // step over ')'

        let mut return_type = None;
        if self.current_token.kind == TokensKind::Colon {
            self.advance(); // step over ':'
            
            if !self.is_type_keyword(self.current_token.kind) {
                return self.error(
                    ErrorKind::ExpectedTypeSignature
                );
            }

            return_type = Some(self.current_token.span);
            self.advance(); // step over return type identifier
        }
        if self.current_token.kind != TokensKind::Lbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbrace, 
                found: self.current_token.kind
            });
        }
        self.advance(); // step over '{'

        let mut body = Vec::new();
        while self.current_token.kind != TokensKind::Rbrace && self.current_token.kind != TokensKind::Eof {
            let stmt_id = self.parse_statement(ast)?;
            body.push(stmt_id);
        }

        if self.current_token.kind != TokensKind::Rbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbrace, 
                found: self.current_token.kind
            });
        }
        let end_span = self.current_token.span;
        self.advance();
        let func_stmt = Statement::Function {
            name: name_span,
            parameters,
            body,
            return_type,
            attribute: function_attribute,
            span: Span {
                start: start_span.start,
                end: end_span.end,
            },
        };

        Ok(ast.add_statement(func_stmt))
    }
    fn parse_if_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError>{
        let start_span = self.current_token.span;
        self.advance(); // advance over if keyword
        // if [x > 4] {
        //    y += 1;
        // }
        // while [x > 4] {
        //    y += 1;
        // }
        if self.current_token.kind != TokensKind::Lbracket {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbracket, 
                found: self.current_token.kind
            });
        }
        self.advance();

        let condition = self.parse_expression(ast, Precedence::Lowest)?;

        if self.current_token.kind != TokensKind::Rbracket {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbracket, 
                found: self.current_token.kind
            });
        }
        self.advance();

        if self.current_token.kind != TokensKind::Lbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbrace, 
                found: self.current_token.kind
            });
        }

        self.advance();
        let mut body = Vec::new();
        while self.current_token.kind != TokensKind::Rbrace && self.current_token.kind != TokensKind::Eof {
            let stmt_id = self.parse_statement(ast)?;
            body.push(stmt_id);
        }
        if self.current_token.kind != TokensKind::Rbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbrace, 
                found: self.current_token.kind
            });
        }
        let mut end_span = self.current_token.span;
        self.advance();

        let mut else_branch = None;
        if self.current_token.kind == TokensKind::KeywordElse {
            self.advance();
            
            if self.current_token.kind == TokensKind::KeywordIf {
                let nested_if = self.parse_if_statement(ast)?;
                else_branch = Some(vec![nested_if]);
            } else if self.current_token.kind == TokensKind::Lbrace {
                self.advance();
                let mut else_stmts = Vec::new();
                while self.current_token.kind != TokensKind::Rbrace && self.current_token.kind != TokensKind::Eof {
                    let stmt_id = self.parse_statement(ast)?;
                    else_stmts.push(stmt_id);
                }
                if self.current_token.kind != TokensKind::Rbrace {
                    return self.error(ErrorKind::ExpectedToken { 
                        expected: TokensKind::Rbrace, 
                        found: self.current_token.kind
                    });
                }
                end_span = self.current_token.span;
                self.advance(); // step over '}'
                else_branch = Some(else_stmts);
            } else {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Lbrace, 
                    found: self.current_token.kind
                });
            }
        }

        let stmt_id = ast.add_statement(Statement::If {
            condition, 
            then_branch: body, 
            else_branch: else_branch,
            span: Span {
                start: start_span.start,
                end: end_span.end,
            } 
        });
        Ok(stmt_id)
    }
    fn parse_expression(&mut self, ast: &mut Ast,precedence: Precedence) -> Result<ExprId, ParseError>{
        let mut left_expr = match self.current_token.kind {
            TokensKind::Identifier => {
                let expr = ast.add_expression(Expression::Identifier { span: self.current_token.span });
                self.advance();
                expr
            },
            TokensKind::AmperSand => {
                let op_span = self.current_token.span;
                self.advance(); 
                
                let right = self.parse_expression(ast, Precedence::Product)?;
                let end_pos = self.get_expr_span(ast, right).end;

                ast.add_expression(Expression::Unary {
                    operator: OpKind::AddressOf,
                    right,
                    span: Span { start: op_span.start, end: end_pos },
                })
            },
            TokensKind::KeywordTrue => {
                let expr = ast.add_expression(Expression::Literal {
                    kind: LiteralKind::Boolean(true),
                    span: self.current_token.span,
                });
                self.advance();
                expr
            },
            TokensKind::KeywordFalse => {
                let expr = ast.add_expression(Expression::Literal {
                    kind: LiteralKind::Boolean(false),
                    span: self.current_token.span,
                });
                self.advance();
                expr
            },
            TokensKind::IntegerLiteral => {
                let raw_text = self.lexer.get_text(self.current_token.span);
                
                let value = raw_text.parse::<i64>().map_err(|_| ParseError {
                    kind: ErrorKind::InvalidLiteral,
                    span: self.current_token.span,
                })?;
                let expr = ast.add_expression(Expression::Literal {
                    kind: LiteralKind::Integer(value),
                    span: self.current_token.span,
                });
                self.advance();
                expr
            },
            TokensKind::FloatLiteral => {
                let raw_text = self.lexer.get_text(self.current_token.span);
                
                let value = raw_text.parse::<f64>().map_err(|_| ParseError {
                    kind: ErrorKind::InvalidLiteral,
                    span: self.current_token.span,
                })?;         
                let expr = ast.add_expression(Expression::Literal {
                    kind: LiteralKind::Float(value),
                    span: self.current_token.span,
                });
                self.advance();
                expr
            },
            TokensKind::StringLiteral => {
                let expr_id = ast.add_expression(Expression::Literal {
                    kind: LiteralKind::String,
                    span: self.current_token.span,
                });

                if let Some(string_id) = self.current_token.id {
                    ast.resolved_string_literals.insert(expr_id, string_id);
                } else {
                    panic!("Parser Error: String literal missing its token id context.");
                }

                self.advance();
                expr_id
            },
            _ =>return self.error(ErrorKind::ExpectedExpression),
        };

        while self.current_token.kind != TokensKind::Semicolon 
            && precedence < self.current_precedence()
            && self.current_token.kind != TokensKind::Rbracket 
        {
            match self.current_token.kind {
                TokensKind::Lparen => {
                    self.advance(); // Step over '('
                    
                    let mut arguments = Vec::new();
                    if self.current_token.kind != TokensKind::Rparen {
                        loop {
                            let arg = self.parse_expression(ast, Precedence::Lowest)?;
                            arguments.push(arg);
                            
                            if self.current_token.kind == TokensKind::Comma {
                                self.advance(); // Step over ','
                            } else {
                                break;
                            }
                        }
                    }
                    
                    if self.current_token.kind != TokensKind::Rparen {
                        return self.error(ErrorKind::ExpectedToken { 
                            expected: TokensKind::Rparen, 
                            found: self.current_token.kind
                        });
                    }
                    let end_pos = self.current_token.span.end;
                    self.advance(); // Step over ')'
                    
                    let start_pos = self.get_expr_span(ast, left_expr).start;
                    
                    left_expr = ast.add_expression(Expression::Call {
                        callee: left_expr,
                        arguments,
                        span: Span { start: start_pos, end: end_pos },
                    });
                },
                TokensKind::Plus | TokensKind::PipePipe | TokensKind::Minus | TokensKind::Asterisk | TokensKind::Slash |
                TokensKind::EqualEqual | TokensKind::BangEqual | TokensKind::LessThan | 
                TokensKind::LessThanEqual | TokensKind::GreaterThan | TokensKind::GreaterThanEqual |
                TokensKind::AndAnd => {
                    let operator_kind = match self.get_op_kind(self.current_token.kind) {
                        Some(v) => v,
                        None => break,
                    };
                    let operator_precedence = self.current_precedence();
                    self.advance();

                    let right_expr = self.parse_expression(ast, operator_precedence)?;

                    let start_pos = self.get_expr_span(ast, left_expr).start;
                    let end_pos = self.get_expr_span(ast, right_expr).end;

                    left_expr = ast.add_expression(Expression::Binary {
                        operator: operator_kind,
                        left: left_expr,
                        right: right_expr,
                        span: Span { start: start_pos, end: end_pos },
                    });
                },
                _ => break,
            }
        }
        Ok(left_expr)
    }





    fn parse_connect_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        self.advance(); // advance over the hashtag #

        if self.current_token.kind != TokensKind::KeywordConnect {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::KeywordConnect, 
                found: self.current_token.kind
            });
        }
        self.advance(); // advance over 'connect'

        if self.current_token.kind != TokensKind::Identifier && self.current_token.kind != TokensKind::StringLiteral {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Identifier, 
                found: self.current_token.kind
            });
        }
        let base_target = self.current_token.span;
        self.advance(); // advance over the base identifier, for example: #connect (identifier)::some_module

        let mut last_target_span = base_target;
        while self.current_token.kind == TokensKind::ColonArrow {
            self.advance();
            if self.current_token.kind != TokensKind::Identifier {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Identifier, 
                    found: self.current_token.kind
                });
            }
            last_target_span = self.current_token.span;
            self.advance();
        }
        if self.current_token.kind != TokensKind::Semicolon {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Semicolon, 
                found: self.current_token.kind
            });
        }
        let end_span = self.current_token.span;
        self.advance(); // step over ';'

        let stmt_id = ast.add_statement(Statement::Connect {
            target: Span {
                start: base_target.start,
                end: last_target_span.end,
            },
            span: Span {
                start: start_span.start,
                end: end_span.end,
            },
        });
        Ok(stmt_id)
    }




    fn get_expr_span(&self, ast: &Ast, id: ExprId) -> Span {
        match ast.expressions[id.0 as usize] {
            Expression::Literal { span, .. } => span,
            Expression::Identifier { span } => span,
            Expression::Binary { span, .. } => span,
            Expression::Call { span, .. } => span,
            Expression::Unary { span, .. } => span,
        }
    }
    #[inline(always)]
    fn is_type_keyword(&self, kind: TokensKind) -> bool {
        match kind {
            TokensKind::KeywordUint64 | TokensKind::KeywordUint32 | TokensKind::KeywordUint16 | TokensKind::KeywordUint8 |
            TokensKind::KeywordInt64  | TokensKind::KeywordInt32  | TokensKind::KeywordInt16  | TokensKind::KeywordInt8  |
            TokensKind::KeywordFloat64 | TokensKind::KeywordFloat32 | TokensKind::KeywordFloat16 | TokensKind::KeywordFloat8 |
            TokensKind::KeywordChar | TokensKind::KeywordBool | TokensKind::KeywordString => true,
            _ => false,
        }
    }
    fn parse_while_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError>{
        let start_span = self.current_token.span;
        self.advance(); // advance over while keyword

        if self.current_token.kind != TokensKind::Lbracket {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbracket, 
                found: self.current_token.kind
            });
        }
        self.advance();

        let condition = self.parse_expression(ast, Precedence::Lowest)?;

        if self.current_token.kind != TokensKind::Rbracket {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbracket, 
                found: self.current_token.kind
            });
        }
        self.advance();

        if self.current_token.kind != TokensKind::Lbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbrace, 
                found: self.current_token.kind
            });
        }

        self.advance();
        let mut body = Vec::new();
        while self.current_token.kind != TokensKind::Rbrace && self.current_token.kind != TokensKind::Eof {
            let stmt_id = self.parse_statement(ast)?;
            body.push(stmt_id);
        }
        if self.current_token.kind != TokensKind::Rbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbrace, 
                found: self.current_token.kind
            });
        }
        let end_span = self.current_token.span;
        self.advance();

        let stmt_id = ast.add_statement(Statement::While {
            condition, 
            body, 
            span: Span {
                start: start_span.start,
                end: end_span.end,
            } 
        });
        Ok(stmt_id)
    }
    #[inline(always)]
    fn current_precedence(&self) -> Precedence {
        match self.current_token.kind {
            TokensKind::Lparen => Precedence::Call,
            TokensKind::AndAnd => Precedence::LogicalAnd,
            TokensKind::PipePipe => Precedence::LogicalOr,
            
            TokensKind::EqualEqual | TokensKind::BangEqual |
            TokensKind::LessThan   | TokensKind::GreaterThan |
            TokensKind::LessThanEqual | TokensKind::GreaterThanEqual => Precedence::Comparison,
            
            TokensKind::Plus | TokensKind::Minus => Precedence::Sum,
            TokensKind::Asterisk | TokensKind::Slash => Precedence::Product,
            
            _ => Precedence::Lowest,
        }
    }







    #[inline(always)]
    fn is_assignment_operator(&self, kind: TokensKind) -> bool {
        match kind {
            TokensKind::Assign | TokensKind::AddBy | 
            TokensKind::SubtractBy | TokensKind::MultiplyBy | 
            TokensKind::DivideBy => true,
            _ => false,
        }
    }





    fn parse_var_or_dec(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        let mut is_mutable = false;
        let is_init = self.current_token.kind == TokensKind::KeywordInit;
        self.advance(); // advance over init or dec

        if self.current_token.kind == TokensKind::KeywordMutable {
            is_mutable = true;
        }
        if self.current_token.kind != TokensKind::Identifier {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Identifier, 
                found: self.current_token.kind
            });
        }

        let name_span = self.current_token.span;
        self.advance(); // advance over identifier , or "x"

        let mut explicit_type = None;
        if self.current_token.kind == TokensKind::Colon {
            self.advance(); // advance over colon (to explicit type setting)
            if !self.is_type_keyword(self.current_token.kind) {
                return self.error(ErrorKind::ExpectedTypeSignature);
            }
            explicit_type = Some(self.current_token.span);
            self.advance();
        }
        let initializer = if self.current_token.kind == TokensKind::Assign {
            self.advance();
            self.parse_expression(ast, Precedence::Lowest)?
        } else {
            ast.add_expression(Expression::Literal {
                kind: LiteralKind::Integer(0), 
                span: name_span,
            })
        };
        if self.current_token.kind != TokensKind::Semicolon {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Semicolon, 
                found: self.current_token.kind
            });
        }

        let end_span = self.current_token.span;
        self.advance();

        let stmd_id = ast.add_statement(Statement::VarDecl {
            is_init,
            is_mutable,
            name: name_span,
            explicit_type,
            initializer,
            span: Span {
                start: start_span.start,
                end: end_span.end,
            } 
        });
        Ok(stmd_id)
    }

    fn parse_expression_or_assignment(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        
        if self.current_token.kind == TokensKind::Identifier &&
            self.is_assignment_operator(self.peek_token.kind) 
        {
            let target = self.current_token.span;
            self.advance(); // step over identifier, example: (identifier) += 3;


            let operator = match self.get_assign_op_kind(self.current_token.kind) {
                Some(op_type) => op_type,
                None => {
                    return self.error(ErrorKind::InvalidAssignmentOperator);
                },
            };
            self.advance(); // step over operator, example: variable

            let value = self.parse_expression(ast, Precedence::Lowest)?;

            if self.current_token.kind != TokensKind::Semicolon {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Semicolon, 
                    found: self.current_token.kind
                });
            }
            let end_span = self.current_token.span;
            self.advance();

            let stmt_id = ast.add_statement(Statement::Assignment {
                target,
                operator,
                value,
                span: Span {
                    start: start_span.start,
                    end: end_span.end,
                },
            });
            return Ok(stmt_id);
        } 
        

        let expr_id = self.parse_expression(ast, Precedence::Lowest)?;
        if self.current_token.kind != TokensKind::Semicolon {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Semicolon, 
                found: self.current_token.kind
            });
        }
        self.advance(); // step over ';'

        let stmt_id = ast.add_statement(Statement::Expression(expr_id));
        Ok(stmt_id)
    }




    fn parse_return_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let mut value = None;
        let start_span = self.current_token.span;
        self.advance();

        if self.current_token.kind != TokensKind::Semicolon {
            value = Some(self.parse_expression(ast, Precedence::Lowest)?);
        }

        if self.current_token.kind != TokensKind::Semicolon {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Semicolon, 
                found: self.current_token.kind
            });
        }
        let end_span = self.current_token.span;
        self.advance();

        let stmt_id = ast.add_statement(Statement::Return { 
            value, 
            span: Span {
                start: start_span.start,end: end_span.end
            } 
        });
        Ok(stmt_id)
    }

    fn parse_structure_statement(&mut self, ast: &mut Ast) -> Result<StmtId, ParseError> {
        let start_span = self.current_token.span;
        self.advance();

        if self.current_token.kind != TokensKind::Identifier {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Identifier, 
                found: self.current_token.kind
            });
        }

        let struct_name = self.current_token.span;
        self.advance();

        if self.current_token.kind != TokensKind::Lbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Lbrace, 
                found: self.current_token.kind
            });
        }

        self.advance();
        let mut fields: Vec<StructField> = Vec::new();
        while self.current_token.kind != TokensKind::Rbrace && self.current_token.kind != TokensKind::Eof {
            let field_start = self.current_token.span;
            if self.current_token.kind != TokensKind::Identifier {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Identifier, 
                    found: self.current_token.kind
                });
            }
            let field_name = self.current_token.span;
            self.advance();

            if self.current_token.kind != TokensKind::Colon {
                return self.error(ErrorKind::ExpectedToken { 
                    expected: TokensKind::Colon, 
                    found: self.current_token.kind
                });
            }
            self.advance();

            if !self.is_type_keyword(self.current_token.kind) {
                return self.error(ErrorKind::ExpectedTypeSignature);
            }
            let field_type = self.current_token.span;
            let field_end = self.current_token.span;
            self.advance();
            fields.push(StructField {
                name: field_name,
                field_type,
                span: Span { start: field_start.start, end: field_end.end },
            });

            if self.current_token.kind == TokensKind::Comma {
                self.advance(); // Advance over optional trailing comma
            }
        }
        if self.current_token.kind != TokensKind::Rbrace {
            return self.error(ErrorKind::ExpectedToken { 
                expected: TokensKind::Rbrace, 
                found: self.current_token.kind
            });
        }
        let end_span = self.current_token.span;
        self.advance();

        let stmt_id = ast.add_statement(Statement::Structure {
             name: struct_name, fields: fields, span: Span {start: start_span.start, end: end_span.end}
        });
        Ok(stmt_id)
    }
    #[inline(always)]
    fn get_op_kind(&self, kind: TokensKind) -> Option<OpKind> {
        match kind {
            TokensKind::Plus => Some(OpKind::Add),
            TokensKind::Minus => Some(OpKind::Sub),
            TokensKind::Asterisk => Some(OpKind::Mul),
            TokensKind::Slash => Some(OpKind::Div),
            TokensKind::EqualEqual => Some(OpKind::Equal),
            TokensKind::BangEqual => Some(OpKind::NotEqual),
            TokensKind::LessThan => Some(OpKind::Less),
            TokensKind::GreaterThan => Some(OpKind::Greater),
            TokensKind::LessThanEqual => Some(OpKind::LessEqual),
            TokensKind::GreaterThanEqual => Some(OpKind::GreaterEqual),
            TokensKind::AndAnd => Some(OpKind::LogicalAnd),
            TokensKind::PipePipe => Some(OpKind::LogicalOr),
            _ => None,
        }
    }
    #[inline(always)]
    fn get_assign_op_kind(&self, kind: TokensKind) -> Option<OpKind> {
        match kind {
            TokensKind::Assign => Some(OpKind::Assign),
            TokensKind::AddBy => Some(OpKind::AddAssign),
            TokensKind::SubtractBy => Some(OpKind::SubAssign),
            TokensKind::MultiplyBy => Some(OpKind::MulAssign),
            TokensKind::DivideBy => Some(OpKind::DivAssign),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Precedence {
    Lowest,
    LogicalOr,    // ||
    LogicalAnd,   // &&
    Comparison,   // ==, !=, <, >, <=, >=
    Sum,          // +, -
    Product,    // *, /
    Prefix,    // -unary, !unary, &address
    Call,
}