// src/codegen.rs

use crate::ast::{
    BinaryOperator, CaseBranch, Declaration, Expr, ForDirection, PascalType, Statement,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(i64),
    PushReal(f64),       // new: for real numbers
    PushString(String),
    PushRecord, // NEW: push an empty record onto the stack
    Load(String),
    Store(String),
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    LE,
    Jump(usize),
    JumpIfFalse(usize),
    Call(String, usize), // Second field: number of arguments.
    FieldAccess(String),
    // Call instructions.
    
    Ret,
}

pub struct CodeGenerator {
    pub instructions: Vec<Instruction>,
    pub functions: HashMap<String, Statement>, // keys stored in lower-case
}


impl CodeGenerator {

    pub fn process_declarations(&mut self, declarations: &[crate::ast::Declaration]) {
        for decl in declarations {
            if let crate::ast::Declaration::VarDeclaration { names, typ } = decl {
                let default_instr = match typ {
                    crate::ast::PascalType::Integer => Instruction::Push(0),
                    crate::ast::PascalType::Real    => Instruction::PushReal(0.0),
                    crate::ast::PascalType::String  => Instruction::PushString(String::new()),
                    crate::ast::PascalType::Boolean => Instruction::Push(0),
                    &crate::ast::PascalType::RecordType(_) => Instruction::PushRecord,
                };
                for name in names {
                    self.instructions.push(default_instr.clone());
                    self.instructions.push(Instruction::Store(name.clone()));
                }
            }
        }
    }

    pub fn new() -> Self {
        CodeGenerator {
            instructions: Vec::new(),
            functions: HashMap::new(),
        }
    }

    pub fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => self.instructions.push(Instruction::Push(*n)),
            Expr::Real(r) => self.instructions.push(Instruction::PushReal(*r)),
            Expr::StringLiteral(s) => self.instructions.push(Instruction::PushString(s.clone())),
            Expr::Variable(name) => self.instructions.push(Instruction::Load(name.clone())),
            Expr::BinaryOp { left, op, right } => {
                self.gen_expr(left);
                self.gen_expr(right);
                match op {
                    BinaryOperator::Add => self.instructions.push(Instruction::Add),
                    BinaryOperator::Subtract => self.instructions.push(Instruction::Sub),
                    BinaryOperator::Multiply => self.instructions.push(Instruction::Mul),
                    BinaryOperator::Divide => self.instructions.push(Instruction::Div),
                    BinaryOperator::Equal => self.instructions.push(Instruction::Equal),
                }
            },
            Expr::RecordLiteral(fields) => {
                // For each field, first evaluate the field expression,
                // then push the field name as a string.
                // At runtime, a built-in "makerecord" function will assemble the record.
                for (field_name, field_expr) in fields {
                    self.gen_expr(field_expr);
                    self.instructions.push(Instruction::PushString(field_name.clone()));
                }
                // The number of arguments is twice the number of fields (value and name per field).
                self.instructions.push(Instruction::Call("makerecord".to_string(), fields.len() * 2));
            },
            // New: Generate code for field access.
            Expr::FieldAccess { record, field } => {
                // First, evaluate the record expression.
                self.gen_expr(record);
                // Then generate an instruction to access the field.
                self.instructions.push(Instruction::FieldAccess(field.clone()));
            },
            Expr::Call { name, args } => {
                for arg in args {
                    self.gen_expr(arg);
                }
                self.instructions.push(Instruction::Call(name.clone(), args.len()));
            },
        }
    }

    pub fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assignment { variable, expr } => {
                self.gen_expr(expr);
                self.instructions.push(Instruction::Store(variable.clone()));
            },
            Statement::Compound(stmts) => {
                for s in stmts {
                    self.gen_statement(s);
                }
            },
            Statement::For { variable, start, end, direction, body } => {
                // Generate code for the starting expression.
                self.gen_expr(start);
                // Store it in the loop variable.
                self.instructions.push(Instruction::Store(variable.clone()));
            
                // Mark the beginning of the loop.
                let loop_start = self.instructions.len();
            
                // Evaluate the condition: load the current loop variable and the end value.
                self.instructions.push(Instruction::Load(variable.clone()));
                self.gen_expr(end);
                // Use our new LE instruction to check if variable <= end.
                self.instructions.push(Instruction::LE);
                // Jump out of the loop if condition is false.
                let jump_if_false_index = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0)); // placeholder
            
                // Generate the code for the loop body.
                self.gen_statement(body);
                
                // Update the loop variable: for a "to" loop, increment; for a "downto" loop, decrement.
                match direction {
                    crate::ast::ForDirection::To => {
                        self.instructions.push(Instruction::Load(variable.clone()));
                        self.instructions.push(Instruction::Push(1));
                        self.instructions.push(Instruction::Add);
                        self.instructions.push(Instruction::Store(variable.clone()));
                    },
                    crate::ast::ForDirection::DownTo => {
                        self.instructions.push(Instruction::Load(variable.clone()));
                        self.instructions.push(Instruction::Push(1));
                        self.instructions.push(Instruction::Sub);
                        self.instructions.push(Instruction::Store(variable.clone()));
                    },
                }
                
                // Jump back to the start of the loop.
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                if let Instruction::JumpIfFalse(ref mut target) = self.instructions[jump_if_false_index] {
                    *target = loop_end;
                }
            },
            // Inside gen_statement() in src/codegen.rs:

            Statement::Case { expr, branches, else_branch } => {
                // Evaluate the controlling expression and store it in a temporary.
                self.gen_expr(expr);
                self.instructions.push(Instruction::Store("case_temp".to_string()));

                // Vectors to record positions for patching later.
                let mut branch_test_positions: Vec<usize> = Vec::new();
                let mut branch_end_jump_positions: Vec<usize> = Vec::new();

                // Generate code for each branch.
                for branch in branches {
                    // Record the position where this branch's test begins.
                    branch_test_positions.push(self.instructions.len());

                    // Generate code for testing:
                    // Load the temporary value and push the branch label.
                    self.instructions.push(Instruction::Load("case_temp".to_string()));
                    self.gen_expr(&branch.label);
                    // Compare for equality.
                    self.instructions.push(Instruction::Equal);
                    // Insert a JumpIfFalse placeholder; we'll patch this later.
                    let test_jump_pos = self.instructions.len();
                    self.instructions.push(Instruction::JumpIfFalse(0));

                    // Generate the branch's statement.
                    self.gen_statement(&branch.stmt);

                    // After the branch, insert an unconditional jump to the end of the case.
                    let branch_end_jump_pos = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0)); // placeholder
                    branch_end_jump_positions.push(branch_end_jump_pos);
                }

                // Generate code for the else branch if it exists.
                let else_position = self.instructions.len();
                if let Some(else_stmt) = else_branch {
                    self.gen_statement(else_stmt);
                }
                let end_of_case = self.instructions.len();

                // Now patch the JumpIfFalse for each branch test.
                for (i, &test_pos) in branch_test_positions.iter().enumerate() {
                    // The JumpIfFalse is at position: test_pos + 3
                    // (because we pushed: Load, then branch label expression, then Equal, then JumpIfFalse)
                    let jump_if_false_index = test_pos + 3;
                    let target = if i + 1 < branch_test_positions.len() {
                        // If there is a next branch, jump to its test start.
                        branch_test_positions[i + 1]
                    } else {
                        // Otherwise, jump to the start of the else branch.
                        else_position
                    };
                    if let Instruction::JumpIfFalse(ref mut t) = self.instructions[jump_if_false_index] {
                        *t = target;
                    }
                }

                // Patch the unconditional jumps at the end of each branch to jump to the end of the case.
                for &jump_pos in branch_end_jump_positions.iter() {
                    if let Instruction::Jump(ref mut t) = self.instructions[jump_pos] {
                        *t = end_of_case;
                    }
                }
            },

            
            
            Statement::RepeatUntil { body, condition } => {
                // Mark the beginning of the loop.
                let loop_start = self.instructions.len();
                // Generate code for the loop body.
                self.gen_statement(body);
                // Evaluate the condition; its result (0 = false, nonzero = true) is left on the stack.
                self.gen_expr(condition);
                // If the condition is false (i.e. 0), then jump back to loop_start.
                self.instructions.push(Instruction::JumpIfFalse(loop_start));
            },

            Statement::If { condition, then_branch, else_branch } => {
                self.gen_expr(condition);
                let jump_if_false_index = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0));
                self.gen_statement(then_branch);
                let jump_index = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));
                let else_start = self.instructions.len();
                if let Some(else_stmt) = else_branch {
                    self.gen_statement(else_stmt);
                }
                let end = self.instructions.len();
                if let Instruction::JumpIfFalse(ref mut target) = self.instructions[jump_if_false_index] {
                    *target = else_start;
                }
                if let Instruction::Jump(ref mut target) = self.instructions[jump_index] {
                    *target = end;
                }
            },
            Statement::While { condition, body } => {
                let start = self.instructions.len();
                self.gen_expr(condition);
                let jump_if_false_index = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0));
                self.gen_statement(body);
                self.instructions.push(Instruction::Jump(start));
                let end = self.instructions.len();
                if let Instruction::JumpIfFalse(ref mut target) = self.instructions[jump_if_false_index] {
                    *target = end;
                }
            },
            Statement::FunctionDecl { name, params, body } => {
                // Store the function declaration for later inlining.
                self.functions.insert(
                    name.to_lowercase(),
                    Statement::FunctionDecl {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            },
            Statement::ProcedureDecl { name, params, body } => {
                self.functions.insert(
                    name.to_lowercase(),
                    Statement::ProcedureDecl {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            },
            Statement::ProcedureCall { name, args } => {
                let name_lower = name.to_lowercase();
                // Handle built-in procedures (writeln, readln, etc.)
                if name_lower == "writeln"
                    || name_lower == "readln"
                    || name_lower == "fact"
                    || name_lower == "sin"
                    || name_lower == "cos"
                    || name_lower == "uppercase"
                    || name_lower == "addreal"
                    || name_lower == "printreal"
                    || name_lower == "fopen"
                    || name_lower == "fread"
                    || name_lower == "fwrite"
                    || name_lower == "fclose"
                {
                    for arg in args {
                        self.gen_expr(arg);
                    }
                    self.instructions.push(Instruction::Call(name.clone(), args.len()));
                } else if let Some(proc_decl) = self.functions.get(&name_lower).cloned() {
                    // For user-defined procedures/functions:
                    // We assume proc_decl is either ProcedureDecl or FunctionDecl,
                    // which include a parameter list of type Vec<(String, PascalType)>.
                    let (params, body) = match proc_decl {
                        Statement::ProcedureDecl { name: _, params, body } => (params, body),
                        Statement::FunctionDecl { name: _, params, body } => (params, body),
                        _ => panic!("Expected procedure/function declaration"),
                    };
                    if params.len() != args.len() {
                        panic!(
                            "Parameter count mismatch in call to {}: expected {}, got {}",
                            name,
                            params.len(),
                            args.len()
                        );
                    }
                    // Evaluate each actual argument and store it into the corresponding formal parameter name.
                    for (i, (param_name, _)) in params.iter().enumerate() {
                        self.gen_expr(&args[i]);
                        self.instructions.push(Instruction::Store(param_name.clone()));
                    }
                    self.gen_statement(&body);
                } else {
                    panic!("Undefined procedure or function: {}", name);
                }
            },
        }
    }
}
