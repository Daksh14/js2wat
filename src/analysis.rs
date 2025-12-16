use std::collections::HashMap;

use crate::parser::{LetStmtBody, ReturnStmt, Stmt};

pub fn dead_code_pass(stmts: &[Stmt], rt_val: Option<&ReturnStmt>) -> Vec<Stmt> {
    let mut filtered = Vec::new();

    let mut table = Vec::new();

    for stmt in stmts.iter() {
        match stmt {
            Stmt::LetStmt(LetStmtBody { var_name, value }) => {
                table.push((var_name, value));
            }
            _ => (),
        }
    }

    let mut count_table = HashMap::new();

    for (var, binary_stmt) in table {
        let lhs = &binary_stmt.lhs;

        if let Some(x) = count_table.get_mut(&var) {
            *x += 1;
        } else {
            count_table.insert(var, 0);
        }

        if let Some(x) = count_table.get_mut(&lhs) {
            *x += 1;
        };
    }

    if let Some(ReturnStmt::BinaryStmtBody(ret_stmt)) = rt_val {
        let rt_lhs = ret_stmt.lhs.clone();

        if let Some(x) = count_table.get_mut(&rt_lhs) {
            *x += 1;
        }
    }

    for stmt in stmts {
        match stmt {
            Stmt::LetStmt(LetStmtBody { var_name, .. }) => {
                // UNWRAP: we just populated count table so it should exit
                let count = count_table.get(&var_name).unwrap();

                if *count > 0 {
                    filtered.push(stmt.clone())
                }
            }
            _ => filtered.push(stmt.clone()),
        }
    }

    filtered
}
