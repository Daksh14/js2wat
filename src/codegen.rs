use crate::{
    analysis::dead_code_pass,
    lexer::Token,
    parser::{
        BinaryStmtBody, FuncCallStmt, FuncDeclBody, IfStmtBody, LetStmtBody, ReturnStmt, Stmt,
        WhileStmtBody,
    },
};

pub fn wat_gen(parsed: Vec<Stmt>) -> String {
    let mut wat = String::new();

    wat.push_str("(module\n");

    for stmt in parsed {
        match stmt {
            Stmt::FuncDecl(FuncDeclBody {
                func_name,
                arguments,
                return_value,
                block,
            }) => {
                let dead_code_pass = dead_code_pass(&block, return_value.as_ref());
                let local_vars = local_var_wat(extract_local_variables(&dead_code_pass));
                let args = argument_var_wat(arguments);

                let mut is_in_else_stmt = 0;

                let fn_body = block_wat(dead_code_pass, &mut is_in_else_stmt);

                let (rt_val, rt_type) = if let Some(x) = return_value {
                    (return_val_wat(x, &mut is_in_else_stmt), "(result i32)")
                } else {
                    (String::new(), "")
                };

                let wat_template = format!(
                    "(func ${} {} {} {}\n {}\n{})\n(export \"{}\" (func ${}))\n",
                    func_name, args, rt_type, local_vars, fn_body, rt_val, func_name, func_name,
                );

                wat.push_str(&wat_template);
            }
            Stmt::FuncCall(call_stmt) => {
                wat.push_str(
                    format!(
                        "(func $_start (result i32)\n{})\n(export \"_start\" (func $_start))",
                        func_call_wat(call_stmt)
                    )
                    .as_str(),
                );
            }
            _ => break,
        }
    }

    wat.push(')');

    wat
}

fn block_wat(stmts: Vec<Stmt>, is_in_else_stmt: &mut u8) -> String {
    let mut wat = String::new();

    for stmt in stmts {
        match stmt {
            Stmt::RassignStmt(LetStmtBody { var_name, value })
            | Stmt::LetStmt(LetStmtBody { var_name, value }) => {
                let binary_stmt = binary_stmt_wat(value);

                wat.push_str(&binary_stmt);

                wat.push_str(format!("local.set ${}\n", var_name).as_str());
            }

            Stmt::WhileStmt(WhileStmtBody { condition, block }) => {
                let cond = binary_stmt_wat(condition);

                let wat_block = block_wat(block, is_in_else_stmt);

                let loop_stmt = format!("loop\n{}\n{} br_if 0 \n end\n", wat_block, cond);
                wat.push_str(&loop_stmt);
            }
            Stmt::IfStmt(IfStmtBody {
                condition,
                if_block,
                if_block_rt_val,
            }) => {
                let cond = binary_stmt_wat(condition);

                // dont exit the else statement because we're about to enter another one
                // we'll increase the counter below
                let code_block = block_wat(if_block, &mut 0);
                let block_rt_val = if let Some(return_stmt) = if_block_rt_val {
                    return_val_wat(return_stmt, &mut 0)
                } else {
                    String::new()
                };

                let if_stmt = format!(
                    "{}\nif (result i32)\n {}\n{}\n else\n ",
                    cond, code_block, block_rt_val,
                );

                *is_in_else_stmt += 1;

                wat.push_str(if_stmt.as_str());
            }
            _ => break,
        }
    }

    wat
}

fn extract_local_variables(stmts: &[Stmt]) -> Vec<String> {
    let mut vars = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::LetStmt(LetStmtBody { var_name, .. }) => {
                vars.push(var_name.to_string());
            }
            Stmt::WhileStmt(WhileStmtBody { block, .. }) => {
                let nested_extracted = extract_local_variables(block);

                vars.extend(nested_extracted);
            }
            _ => break,
        }
    }

    vars
}

fn return_val_wat(stmt: ReturnStmt, is_in_else_stmt: &mut u8) -> String {
    let mut val = match stmt {
        ReturnStmt::BinaryStmtBody(binary_stmt) => binary_stmt_wat(binary_stmt),
        ReturnStmt::FuncCallStmt(func_call) => func_call_wat(func_call),
    };

    for _ in 0..(*is_in_else_stmt) {
        val.push_str("\nend\n");
    }

    val
}

fn func_call_wat(stmt: FuncCallStmt) -> String {
    let FuncCallStmt {
        function_name,
        arguments,
    } = stmt;

    let mut call = String::new();

    for arg in arguments {
        match arg {
            Stmt::BinaryStmt(binary_stmt) => call.push_str(binary_stmt_wat(binary_stmt).as_str()),
            Stmt::FuncCall(func_call) => {
                call.push_str(func_call_wat(func_call).as_str());
            }
            _ => break,
        }
    }

    call.push_str(format!("call ${}\n", function_name).as_str());

    call
}

fn binary_stmt_wat(stmt: BinaryStmtBody) -> String {
    let mut temp = String::new();

    let BinaryStmtBody { lhs, rhs, op } = stmt;

    temp.push_str(&local_or_const_wat(&lhs));

    if let Some(right_hand) = rhs {
        temp.push_str(&local_or_const_wat(&right_hand));
        if let Some(operation) = op {
            let wat_op = op_wat(operation);
            temp.push_str(wat_op);
        }
    }

    temp
}

fn argument_var_wat(stmts: Vec<Stmt>) -> String {
    let mut wat = String::new();

    for stmt in stmts {
        if let Stmt::BinaryStmt(BinaryStmtBody { lhs, .. }) = stmt {
            wat.push_str(format!("(param ${} i32) ", lhs).as_str());
        }
    }

    wat
}

fn local_var_wat(vars: Vec<String>) -> String {
    let mut wat = String::new();

    for var in vars {
        wat.push_str(format!("(local ${} i32) ", var).as_str())
    }

    wat
}

fn check_if_literal_i32(text: &str) -> bool {
    text.chars().all(char::is_numeric)
}

fn local_or_const_wat(text: &str) -> String {
    if check_if_literal_i32(text) {
        format!("i32.const {}\n", text)
    } else {
        format!("local.get ${}\n", text)
    }
}

fn op_wat(token: Token) -> &'static str {
    match token {
        Token::Add => "i32.add\n",
        Token::Mul => "i32.mul\n",
        Token::GreaterThan => "i32.lt_u\n",
        Token::LessThan => "i32.gt_u\n",
        Token::Sub => "i32.sub\n",
        Token::DoubleEq => "i32.eq\n",
        Token::Percent => "i32.rem_u\n",
        Token::NotEq => "i32.ne\n",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::parser::{
        BinaryStmtBody, FuncCallStmt, FuncDeclBody, IfStmtBody, LetStmtBody, ReturnStmt, Stmt,
        WhileStmtBody,
    };

    #[test]
    fn gen_simple_return_function() {
        let ast = vec![Stmt::FuncDecl(FuncDeclBody {
            func_name: "main".into(),
            arguments: vec![],
            block: vec![],
            return_value: Some(ReturnStmt::BinaryStmtBody(BinaryStmtBody {
                lhs: "1".into(),
                rhs: None,
                op: None,
            })),
        })];

        let wat = wat_gen(ast);

        assert!(wat.contains("(func $main"));
        assert!(wat.contains("i32.const 1"));
        assert!(wat.contains("(export \"main\""));
    }

    #[test]
    fn gen_binary_addition() {
        let ast = vec![Stmt::FuncDecl(FuncDeclBody {
            func_name: "add".into(),
            arguments: vec![],
            block: vec![],
            return_value: Some(ReturnStmt::BinaryStmtBody(BinaryStmtBody {
                lhs: "1".into(),
                rhs: Some("2".into()),
                op: Some(Token::Add),
            })),
        })];

        let wat = wat_gen(ast);

        assert!(wat.contains("i32.const 1"));
        assert!(wat.contains("i32.const 2"));
        assert!(wat.contains("i32.add"));
    }

    #[test]
    fn gen_local_variable_assignment() {
        let ast = vec![Stmt::FuncDecl(FuncDeclBody {
            func_name: "main".into(),
            arguments: vec![],
            block: vec![Stmt::LetStmt(LetStmtBody {
                var_name: "x".into(),
                value: BinaryStmtBody {
                    lhs: "10".into(),
                    rhs: None,
                    op: None,
                },
            })],
            return_value: None,
        })];

        let wat = wat_gen(ast);

        assert!(wat.contains("(local $x i32)"));
        assert!(wat.contains("i32.const 10"));
        assert!(wat.contains("local.set $x"));
    }

    #[test]
    fn gen_function_call() {
        let ast = vec![Stmt::FuncDecl(FuncDeclBody {
            func_name: "main".into(),
            arguments: vec![],
            block: vec![Stmt::FuncCall(FuncCallStmt {
                function_name: "foo".into(),
                arguments: Vec::new(),
            })],
            return_value: Some(ReturnStmt::BinaryStmtBody(BinaryStmtBody {
                lhs: "5".into(),
                rhs: None,
                op: None,
            })),
        })];

        let wat = wat_gen(ast);

        assert!(wat.contains("i32.const 5"));
        assert!(wat.contains("call $foo"));
    }

    #[test]
    fn gen_while_loop() {
        let ast = vec![Stmt::FuncDecl(FuncDeclBody {
            func_name: "loop_fn".into(),
            arguments: vec![],
            block: vec![Stmt::WhileStmt(WhileStmtBody {
                condition: BinaryStmtBody {
                    lhs: "x".into(),
                    rhs: Some("10".into()),
                    op: Some(Token::LessThan),
                },
                block: vec![],
            })],
            return_value: None,
        })];

        let wat = wat_gen(ast);

        assert!(wat.contains("loop"));
        assert!(wat.contains("i32.gt_u"));
        assert!(wat.contains("br_if 0"));
    }

    #[test]
    fn gen_if_statement_with_return() {
        let ast = vec![Stmt::FuncDecl(FuncDeclBody {
            func_name: "cond".into(),
            arguments: vec![],
            block: vec![Stmt::IfStmt(IfStmtBody {
                condition: BinaryStmtBody {
                    lhs: "1".into(),
                    rhs: Some("1".into()),
                    op: Some(Token::DoubleEq),
                },
                if_block: vec![],
                if_block_rt_val: Some(ReturnStmt::BinaryStmtBody(BinaryStmtBody {
                    lhs: "42".into(),
                    rhs: None,
                    op: None,
                })),
            })],
            return_value: None,
        })];

        let wat = wat_gen(ast);

        assert!(wat.contains("if (result i32)"));
        assert!(wat.contains("i32.eq"));
        assert!(wat.contains("i32.const 42"));
        assert!(wat.contains("end"));
    }
}
