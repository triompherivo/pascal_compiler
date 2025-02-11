// src/parser.rs

use crate::ast::{
    Expr, Statement, BinaryOperator, Declaration, Program, PascalType, ForDirection, CaseBranch,
};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }
    
    pub fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }
    
    pub fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF);
        self.pos += 1;
        tok
    }
    
    fn expect(&mut self, expected: &Token) {
        let tok = self.advance();
        if &tok != expected {
            panic!("Expected {:?}, found {:?}", expected, tok);
        }
    }
    
    /// Parse the top-level program.
    /// Format: optional var section, then main statement, then a dot.
    pub fn parse_program(&mut self) -> Program {
        // Vectors to collect different kinds of declarations.
        let mut var_decls: Vec<Declaration> = Vec::new();
        let mut proc_decls: Vec<Statement> = Vec::new();
    
        // Loop while the next token is Var, Function, or Procedure.
        loop {
            match self.peek() {
                Token::Var => {
                    // Parse variable declarations.
                    let decls = self.parse_var_section();
                    var_decls.extend(decls);
                },
                Token::Function | Token::Procedure => {
                    // Parse function or procedure declaration.
                    let decl = self.parse_decl();
                    proc_decls.push(decl);
                },
                _ => break,
            }
        }
    
        // The next token should be the beginning of the main block.
        // Parse the main statement.
        let main_stmt = self.parse_statement();
        self.expect(&Token::Dot);
    
        // Now, combine the procedure declarations and main statement.
        // One approach: if there are any procedure declarations, prepend them in a compound statement.
        let combined_main = if proc_decls.is_empty() {
            main_stmt
        } else {
            let mut stmts = proc_decls;
            stmts.push(main_stmt);
            Statement::Compound(stmts)
        };
    
        // Construct and return the Program AST.
        Program {
            declarations: var_decls,
            main: combined_main,
        }
    }
    
    
    /// Parse a variable declaration section.
    /// Syntax:
    ///   var
    ///      id1, id2: integer;
    ///      id3: string;
    fn parse_var_section(&mut self) -> Vec<Declaration> {
        self.expect(&Token::Var);
        let mut decls = Vec::new();
        while let Token::Identifier(_) = self.peek() {
            let names = self.parse_identifier_list();
            self.expect(&Token::Colon);
            let typ = self.parse_type();
            self.expect(&Token::Semicolon);
            decls.push(Declaration::VarDeclaration { names, typ });
        }
        decls
    }
    
    /// Parse a list of identifiers separated by commas.
    fn parse_identifier_list(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        if let Token::Identifier(n) = self.advance() {
            names.push(n);
        } else {
            panic!("Expected identifier in variable declaration");
        }
        while let Token::Comma = self.peek() {
            self.advance(); // consume comma
            if let Token::Identifier(n) = self.advance() {
                names.push(n);
            } else {
                panic!("Expected identifier after comma");
            }
        }
        names
    }
    
    /// Parse a type and return a PascalType.
    fn parse_type(&mut self) -> PascalType {
        match self.peek() {
            Token::Identifier(_) => {
                if let Token::Identifier(type_name) = self.advance() {
                    match type_name.to_lowercase().as_str() {
                        "integer" => PascalType::Integer,
                        "real"    => PascalType::Real,
                        "string"  => PascalType::String,
                        "boolean" => PascalType::Boolean,
                        _ => panic!("Unknown type: {}", type_name),
                    }
                } else {
                    unreachable!()
                }
            },
            Token::Record => {
                self.advance(); // consume 'record'
                // Allow an optional semicolon right after "record"
                if let Token::Semicolon = self.peek() {
                    self.advance();
                }
                let mut fields = Vec::new();
                // Parse field declarations until we see 'End'
                while let Token::Identifier(_) = self.peek() {
                    let field_names = self.parse_identifier_list();
                    self.expect(&Token::Colon);
                    let field_type = self.parse_type();
                    self.expect(&Token::Semicolon);
                    for name in field_names {
                        fields.push((name, field_type.clone()));
                    }
                }
                self.expect(&Token::End); // consume 'end'
                PascalType::RecordType(fields)
            },
            _ => panic!("Expected type identifier, found {:?}", self.peek()),
        }
    }
    
    
    
    pub fn parse_statement(&mut self) -> Statement {
        match self.peek() {
            Token::Begin => self.parse_compound(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Repeat => self.parse_repeat_until(),
            Token::Case => self.parse_case(),
            Token::Function | Token::Procedure => self.parse_decl(),
            Token::Identifier(_) | Token::Writeln | Token::Readln => self.parse_assignment_or_proc_call(),
            _ => panic!("Unexpected token in statement: {:?}", self.peek()),
        }
    }
    
    fn parse_compound(&mut self) -> Statement {
        self.expect(&Token::Begin);
        let mut stmts = Vec::new();
        while self.peek() != &Token::End {
            let stmt = self.parse_statement();
            stmts.push(stmt);
            if let Token::Semicolon = self.peek() {
                self.advance();
            }
        }
        self.expect(&Token::End);
        Statement::Compound(stmts)
    }
    
    fn parse_if(&mut self) -> Statement {
        self.expect(&Token::If);
        let cond = self.parse_expr();
        self.expect(&Token::Then);
        let then_branch = Box::new(self.parse_statement());
        let else_branch = if let Token::Else = self.peek() {
            self.advance();
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };
        Statement::If { condition: cond, then_branch, else_branch }
    }
    
    fn parse_while(&mut self) -> Statement {
        self.expect(&Token::While);
        let cond = self.parse_expr();
        self.expect(&Token::Do);
        let body = Box::new(self.parse_statement());
        Statement::While { condition: cond, body }
    }
    
    fn parse_for(&mut self) -> Statement {
        self.expect(&Token::For);
        let var = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected loop variable after 'for'"); };
        self.expect(&Token::ColonEqual);
        let start_expr = self.parse_expr();
        let direction = match self.advance() {
            Token::To => ForDirection::To,
            Token::Downto => ForDirection::DownTo,
            token => panic!("Expected 'to' or 'downto', found {:?}", token),
        };
        let end_expr = self.parse_expr();
        self.expect(&Token::Do);
        let body = Box::new(self.parse_statement());
        Statement::For {
            variable: var,
            start: Box::new(start_expr),
            end: Box::new(end_expr),
            direction,
            body,
        }
    }
    
    fn parse_repeat_until(&mut self) -> Statement {
        self.expect(&Token::Repeat);
        let mut stmts = Vec::new();
        while self.peek() != &Token::Until {
            let stmt = self.parse_statement();
            stmts.push(stmt);
            if let Token::Semicolon = self.peek() {
                self.advance();
            }
        }
        self.expect(&Token::Until);
        let cond = self.parse_expr();
        Statement::RepeatUntil { body: Box::new(Statement::Compound(stmts)), condition: cond }
    }
    
    fn parse_case(&mut self) -> Statement {
        self.expect(&Token::Case);
        let expr = self.parse_expr();
        self.expect(&Token::Of);
        let mut branches = Vec::new();
        while let Token::Number(_) | Token::Identifier(_) | Token::StringLiteral(_) = self.peek() {
            let label = self.parse_expr();
            self.expect(&Token::Colon);
            let stmt = self.parse_statement();
            branches.push(crate::ast::CaseBranch { label, stmt });
            if let Token::Semicolon = self.peek() {
                self.advance();
            }
        }
        let else_branch = if let Token::Else = self.peek() {
            self.advance();
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };
        if let Token::Semicolon = self.peek() {
            self.advance();
        }
        self.expect(&Token::End);
        Statement::Case { expr, branches, else_branch }
    }
    
    fn parse_decl(&mut self) -> Statement {
        match self.peek() {
            Token::Function => self.parse_function_decl(),
            Token::Procedure => self.parse_procedure_decl(),
            _ => panic!("Expected function or procedure declaration"),
        }
    }
    
    fn parse_function_decl(&mut self) -> Statement {
        self.expect(&Token::Function);
        let name = if let Token::Identifier(n) = self.advance() {
            n
        } else {
            panic!("Expected function name");
        };
        let params = self.parse_param_list(); // now returns Vec<(String, PascalType)>
        self.expect(&Token::Colon);
        self.advance(); // skip return type (for simplicity, we ignore it)
        self.expect(&Token::Semicolon);
        let body = Box::new(self.parse_statement());
        self.expect(&Token::Semicolon);
        Statement::FunctionDecl { name, params, body }
    }
    
    fn parse_procedure_decl(&mut self) -> Statement {
        self.expect(&Token::Procedure);
        let name = if let Token::Identifier(n) = self.advance() {
            n
        } else {
            panic!("Expected procedure name");
        };
        let params = self.parse_param_list();
        self.expect(&Token::Semicolon);
        let body = Box::new(self.parse_statement());
        self.expect(&Token::Semicolon);
        Statement::ProcedureDecl { name, params, body }
    }
    
    /// Parse a parameter list that may be empty.
    fn parse_param_list(&mut self) -> Vec<(String, PascalType)> {
        let mut params = Vec::new();
        if let Token::LParen = self.peek() {
            self.expect(&Token::LParen);
            if let Token::RParen = self.peek() {
                self.expect(&Token::RParen);
                return params;
            }
            // Parse first parameter: identifier, colon, type
            if let Token::Identifier(n) = self.advance() {
                let param_name = n;
                self.expect(&Token::Colon);
                let param_type = self.parse_type();
                params.push((param_name, param_type));
            } else {
                panic!("Expected parameter name after '('");
            }
            // Parse remaining parameters separated by comma or semicolon.
            while let Token::Comma | Token::Semicolon = self.peek() {
                self.advance(); // consume separator
                if let Token::Identifier(n) = self.advance() {
                    let param_name = n;
                    self.expect(&Token::Colon);
                    let param_type = self.parse_type();
                    params.push((param_name, param_type));
                } else {
                    panic!("Expected parameter name after separator");
                }
            }
            self.expect(&Token::RParen);
        }
        params
    }
        
    /// Parse an assignment or a procedure call.
    fn parse_assignment_or_proc_call(&mut self) -> Statement {
        let token = self.advance();
        let name = match token {
            Token::Identifier(n) => n,
            Token::Writeln => "writeln".to_string(),
            Token::Readln => "readln".to_string(),
            _ => panic!("Expected identifier or built-in procedure, got {:?}", token),
        };
        match self.peek() {
            Token::ColonEqual => {
                self.advance(); // consume ':='
                let expr = self.parse_expr();
                Statement::Assignment { variable: name, expr }
            },
            Token::LParen => {
                self.advance(); // consume '('
                let mut args = Vec::new();
                if self.peek() != &Token::RParen {
                    args.push(self.parse_expr());
                    while let Token::Comma = self.peek() {
                        self.advance();
                        args.push(self.parse_expr());
                    }
                }
                self.expect(&Token::RParen);
                Statement::ProcedureCall { name, args }
            },
            _ => Statement::ProcedureCall { name, args: Vec::new() },
        }
    }
    
    pub fn parse_expr(&mut self) -> Expr {
        self.parse_equality()
    }
    
    fn parse_equality(&mut self) -> Expr {
        let mut node = self.parse_addition();
        while let Token::Equal = self.peek() {
            self.advance();
            let right = self.parse_addition();
            node = Expr::BinaryOp {
                left: Box::new(node),
                op: BinaryOperator::Equal,
                right: Box::new(right),
            };
        }
        node
    }
    
    fn parse_addition(&mut self) -> Expr {
        let mut node = self.parse_term();
        while let Token::Plus | Token::Minus = self.peek() {
            let op = match self.advance() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_term();
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(right),
            };
        }
        node
    }
    
    fn parse_term(&mut self) -> Expr {
        let mut node = self.parse_factor();
        while let Token::Star | Token::Slash = self.peek() {
            let op = match self.advance() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                _ => unreachable!(),
            };
            let right = self.parse_factor();
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(right),
            };
        }
        node
    }
    
    fn parse_factor(&mut self) -> Expr {
        // First, parse a primary expression.
        let mut node = match self.peek() {
            Token::Number(_) => {
                if let Token::Number(s) = self.advance() {
                    if s.contains('.') {
                        Expr::Real(s.parse::<f64>().unwrap())
                    } else {
                        Expr::Number(s.parse::<i64>().unwrap())
                    }
                } else { unreachable!() }
            },
            Token::StringLiteral(_) => {
                if let Token::StringLiteral(s) = self.advance() {
                    Expr::StringLiteral(s)
                } else { unreachable!() }
            },
            Token::Identifier(_) | Token::Writeln | Token::Readln => {
                let token = self.advance();
                let name = match token {
                    Token::Identifier(n) => n,
                    Token::Writeln => "writeln".to_string(),
                    Token::Readln => "readln".to_string(),
                    _ => unreachable!(),
                };
                if let Token::LParen = self.peek() {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        args.push(self.parse_expr());
                        while let Token::Comma = self.peek() {
                            self.advance();
                            args.push(self.parse_expr());
                        }
                    }
                    self.expect(&Token::RParen);
                    Expr::Call { name, args }
                } else {
                    Expr::Variable(name)
                }
            },
            Token::LParen => {
                self.advance(); // consume '('
                let expr = self.parse_expr();
                self.expect(&Token::RParen);
                expr
            },
            Token::Record => {
                // Parse a record literal: record( field: expr; ... )
                self.advance(); // consume "record" keyword
                self.expect(&Token::LParen);
                let mut fields = Vec::new();
                while self.peek() != &Token::RParen {
                    let field_name = if let Token::Identifier(n) = self.advance() {
                        n
                    } else {
                        panic!("Expected field name in record literal");
                    };
                    self.expect(&Token::Colon);
                    let field_expr = self.parse_expr();
                    fields.push((field_name, field_expr));
                    if let Token::Semicolon = self.peek() {
                        self.advance();
                    }
                }
                self.expect(&Token::RParen);
                Expr::RecordLiteral(fields)
            },
            _ => panic!("Unexpected token in expression: {:?}", self.peek()),
        };
    
        // Now handle field access (e.g., p.name)
        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance(); // consume '.'
                    
                    match self.peek() {
                        Token::Identifier(field_name) => {
                            let field = field_name.clone();
                            self.advance(); // consume the field name
                            node = Expr::FieldAccess {
                                record: Box::new(node),
                                field,
                            };
                        },
                        /* Token::EOF => {
                            panic!("Expected field name after '.', reached end of file ");
                        }, */
                        other => {
                            panic!("Expected field name after '.', found {:?}", other);
                        }
                    }
                }
                _ => break,
            }
        }
        node
    }
        
}
