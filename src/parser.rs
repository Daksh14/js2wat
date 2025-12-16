use std::iter::Peekable;

use crate::lexer::Token;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Stmt {
    FuncDecl(FuncDeclBody),
    WhileStmt(WhileStmtBody),
    IfStmt(IfStmtBody),
    LetStmt(LetStmtBody),
    RassignStmt(LetStmtBody),
    BinaryStmt(BinaryStmtBody),
    FuncCall(FuncCallStmt),
}

#[derive(Debug)]
pub enum ReturnStmt {
    BinaryStmtBody(BinaryStmtBody),
    FuncCallStmt(FuncCallStmt),
}

#[derive(Debug)]
pub struct FuncCallStmt {
    pub function_name: String,
    pub arguments: Vec<Stmt>,
}

#[derive(Debug)]
pub struct IfStmtBody {
    pub condition: BinaryStmtBody,
    pub if_block: Vec<Stmt>,
    pub if_block_rt_val: Option<ReturnStmt>,
}

#[derive(Debug)]
pub struct LetStmtBody {
    pub var_name: String,
    pub value: BinaryStmtBody,
}

#[derive(Debug)]
pub struct WhileStmtBody {
    pub condition: BinaryStmtBody,
    pub block: Vec<Stmt>,
}

#[derive(Debug)]
pub struct FuncDeclBody {
    pub func_name: String,
    pub arguments: Vec<Stmt>,
    pub return_value: Option<ReturnStmt>,
    pub block: Vec<Stmt>,
}

#[derive(Debug)]
pub struct BinaryStmtBody {
    pub lhs: String,
    pub rhs: Option<String>,
    pub op: Option<Token>,
}

pub fn parse(token_stream: Vec<Token>) -> Vec<Stmt> {
    let mut peekable = token_stream.iter().peekable();

    let mut tree = Vec::new();

    while let Some(token) = peekable.peek() {
        match token {
            Token::Function => {
                peekable.next();

                exhaust_whitespace(&mut peekable);

                if let Some(Token::Literal(fn_name)) = peekable.peek() {
                    peekable.next();

                    assert_token(&mut peekable, &Token::ParenOpen);

                    let fn_args = parse_fn_arguments(&mut peekable);

                    exhaust_whitespace(&mut peekable);

                    assert_token(&mut peekable, &Token::CurlyOpen);
                    let (parsed_block, ret_val) = parse_block(&mut peekable);

                    let fn_body = FuncDeclBody {
                        func_name: fn_name.clone(),
                        arguments: fn_args,
                        return_value: ret_val,
                        block: parsed_block,
                    };

                    tree.push(Stmt::FuncDecl(fn_body));
                } else {
                    panic!("Expected function name");
                }
            }
            Token::Literal(potential_func) => {
                peekable.next();
                assert_token(&mut peekable, &Token::ParenOpen);

                let fn_args = parse_fn_arguments(&mut peekable);

                let fn_call = FuncCallStmt {
                    function_name: potential_func.clone(),
                    arguments: fn_args,
                };

                tree.push(Stmt::FuncCall(fn_call));

                assert_token(&mut peekable, &Token::SemiColon);
            }
            Token::Comment => {
                peekable.next();

                while let Some(token) = peekable.peek() {
                    if let Token::NewLine = token {
                        break;
                    } else {
                        peekable.next();
                    }
                }
            }
            _ => {
                peekable.next();
            }
        }
    }

    tree
}

fn parse_block<'a, I: Iterator<Item = &'a Token>>(
    peekable: &mut Peekable<I>,
) -> (Vec<Stmt>, Option<ReturnStmt>) {
    let mut blocks = Vec::new();
    let mut ret_val = None;

    while let Some(token) = peekable.peek() {
        match token {
            Token::Let => {
                peekable.next();

                exhaust_whitespace(peekable);

                if let Some(Token::Literal(x)) = peekable.peek() {
                    peekable.next();

                    exhaust_whitespace(peekable);
                    assert_token(peekable, &Token::Eq);
                    exhaust_whitespace(peekable);

                    let let_stmt = LetStmtBody {
                        var_name: x.clone(),
                        value: parse_binary_stmt(peekable, None),
                    };

                    assert_token(peekable, &Token::SemiColon);

                    blocks.push(Stmt::LetStmt(let_stmt));
                } else {
                    panic!("Execpted literal after eq in let stmt");
                }
            }
            Token::While => {
                peekable.next();

                exhaust_whitespace(peekable);
                assert_token(peekable, &Token::ParenOpen);

                let condition = parse_binary_stmt(peekable, None);

                assert_token(peekable, &Token::ParenClose);

                exhaust_whitespace(peekable);
                let while_stmt = WhileStmtBody {
                    condition,
                    block: parse_block(peekable).0,
                };

                blocks.push(Stmt::WhileStmt(while_stmt));
            }
            Token::Literal(x) => {
                peekable.next();

                exhaust_whitespace(peekable);

                match peekable.peek() {
                    Some(Token::Eq) => {
                        peekable.next();

                        exhaust_whitespace(peekable);
                        let stmt = Stmt::RassignStmt(LetStmtBody {
                            var_name: x.clone(),
                            value: parse_binary_stmt(peekable, None),
                        });

                        blocks.push(stmt);

                        assert_token(peekable, &Token::SemiColon);
                    }
                    Some(Token::ParenOpen) => {
                        peekable.next();
                        let args = parse_fn_arguments(peekable);

                        blocks.push(Stmt::FuncCall(FuncCallStmt {
                            function_name: x.clone(),
                            arguments: args,
                        }));

                        assert_token(peekable, &Token::SemiColon);
                    }
                    _ => {
                        println!("{:?}", x);
                        println!("{:?}", peekable.peek())
                    } // _ => panic!("Unexpected token"),
                }
            }
            Token::If => {
                peekable.next();
                exhaust_whitespace(peekable);
                assert_token(peekable, &Token::ParenOpen);

                let expr = parse_binary_stmt(peekable, None);

                assert_token(peekable, &Token::ParenClose);

                exhaust_whitespace(peekable);

                let (body, ret_val) = parse_block(peekable);

                blocks.push(Stmt::IfStmt(IfStmtBody {
                    condition: expr,
                    if_block: body,
                    if_block_rt_val: ret_val,
                }));
            }
            Token::Return => {
                peekable.next();

                exhaust_whitespace(peekable);

                if let Some(Token::Literal(val)) = peekable.peek() {
                    peekable.next();

                    if let Some(Token::ParenOpen) = peekable.peek() {
                        peekable.next();

                        let fn_call = FuncCallStmt {
                            function_name: val.to_owned(),
                            arguments: parse_fn_arguments(peekable),
                        };

                        ret_val = Some(ReturnStmt::FuncCallStmt(fn_call));
                    } else {
                        let stmt = parse_binary_stmt(peekable, Some(val));

                        ret_val = Some(ReturnStmt::BinaryStmtBody(stmt));
                    }
                };

                assert_token(peekable, &Token::SemiColon);

                break;
            }
            Token::CurlyClose => break,
            _ => {
                peekable.next();
            }
        }
    }

    peekable.next();

    (blocks, ret_val)
}

fn parse_fn_arguments<'a, I: Iterator<Item = &'a Token>>(peekable: &mut Peekable<I>) -> Vec<Stmt> {
    let mut fn_args = Vec::new();

    while let Some(token) = peekable.peek() {
        match token {
            Token::Literal(val) => {
                peekable.next();
                exhaust_whitespace(peekable);

                if let Some(Token::ParenOpen) = peekable.peek() {
                    peekable.next();

                    let fn_call = FuncCallStmt {
                        function_name: val.to_owned(),
                        arguments: parse_fn_arguments(peekable),
                    };

                    fn_args.push(Stmt::FuncCall(fn_call));
                } else {
                    let stmt = parse_binary_stmt(peekable, Some(val));
                    fn_args.push(Stmt::BinaryStmt(stmt));
                }
            }
            Token::Comma | Token::WhiteSpace => {
                peekable.next();
            }
            Token::ParenClose => {
                peekable.next();

                break;
            }
            _ => panic!("Unexpected token in function decl"),
        }
    }

    fn_args
}

fn parse_binary_stmt<'a, I: Iterator<Item = &'a Token>>(
    peekable: &mut Peekable<I>,
    lhs: Option<&String>,
) -> BinaryStmtBody {
    fn parse_op_rhs<'b, I: Iterator<Item = &'b Token>>(
        x: &str,
        peekable: &mut Peekable<I>,
    ) -> BinaryStmtBody {
        // if its a single value
        if let Some(Token::Comma) | Some(Token::ParenClose) = peekable.peek() {
            return BinaryStmtBody {
                lhs: x.to_string(),
                rhs: None,
                op: None,
            };
        }

        exhaust_whitespace(peekable);

        let op = if let Some(token) = peekable.peek() {
            match token {
                Token::SemiColon => {
                    return BinaryStmtBody {
                        lhs: x.to_string(),
                        rhs: None,
                        op: None,
                    }
                }
                Token::Exclaim => {
                    peekable.next();

                    assert_token(peekable, &Token::Eq);

                    Some(Token::NotEq)
                }
                Token::Eq => {
                    peekable.next();

                    if let Some(Token::Eq) = peekable.peek() {
                        peekable.next();

                        Some(Token::DoubleEq)
                    } else {
                        Some(Token::Eq)
                    }
                }
                _ => {
                    let op = peekable.next().unwrap();

                    Some(op.clone())
                }
            }
        } else {
            None
        };

        exhaust_whitespace(peekable);

        let rhs = if let Some(Token::Literal(y)) = peekable.peek() {
            peekable.next();

            Some(y.clone())
        } else {
            None
        };

        BinaryStmtBody {
            lhs: x.to_string(),
            rhs,
            op,
        }
    }

    if let Some(Token::Literal(x)) = peekable.peek() {
        peekable.next();

        return parse_op_rhs(x, peekable);
    } else if let Some(passed_lhs) = lhs {
        return parse_op_rhs(passed_lhs, peekable);
    }

    panic!("Invalid binary stmt");
}

fn exhaust_whitespace<'a, I: Iterator<Item = &'a Token>>(
    peekable: &mut Peekable<I>,
) -> &mut Peekable<I> {
    // skip both space and tabs
    while let Some(Token::WhiteSpace) | Some(Token::Tab) = peekable.peek() {
        peekable.next();
    }

    peekable
}

fn assert_token<'a, I: Iterator<Item = &'a Token>>(peekable: &mut Peekable<I>, token: &Token) {
    if let Some(assert_token) = peekable.next() {
        if assert_token != token {
            panic!("Expected: {:?} token, found: {:?} ", token, assert_token);
        }
    } else {
        panic!("Expected token found None");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;

    fn t(s: &str) -> Token {
        Token::Literal(s.to_string())
    }

    #[test]
    fn parses_empty_input() {
        let ast = parse(vec![]);
        assert!(ast.is_empty());
    }

    #[test]
    fn parses_function_declaration_no_body() {
        let tokens = vec![
            Token::Function,
            Token::WhiteSpace,
            t("foo"),
            Token::ParenOpen,
            Token::ParenClose,
            Token::CurlyOpen,
            Token::CurlyClose,
        ];

        let ast = parse(tokens);

        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Stmt::FuncDecl(body) => {
                assert_eq!(body.func_name, "foo");
                assert!(body.arguments.is_empty());
                assert!(body.block.is_empty());
                assert!(body.return_value.is_none());
            }
            _ => panic!("Expected FuncDecl"),
        }
    }

    #[test]
    fn parses_function_with_let_and_return() {
        let tokens = vec![
            Token::Function,
            Token::WhiteSpace,
            t("f"),
            Token::ParenOpen,
            Token::ParenClose,
            Token::CurlyOpen,
            Token::Let,
            Token::WhiteSpace,
            t("x"),
            Token::WhiteSpace,
            Token::Eq,
            Token::WhiteSpace,
            t("1"),
            Token::SemiColon,
            Token::Return,
            Token::WhiteSpace,
            t("x"),
            Token::SemiColon,
            Token::CurlyClose,
        ];

        let ast = parse(tokens);

        match &ast[0] {
            Stmt::FuncDecl(body) => {
                assert_eq!(body.block.len(), 1);

                match &body.block[0] {
                    Stmt::LetStmt(let_stmt) => {
                        assert_eq!(let_stmt.var_name, "x");
                        assert_eq!(let_stmt.value.lhs, "1");
                    }
                    _ => panic!("Expected LetStmt"),
                }

                match &body.return_value {
                    Some(ReturnStmt::BinaryStmtBody(bin)) => {
                        assert_eq!(bin.lhs, "x");
                        assert!(bin.op.is_none());
                    }
                    _ => panic!("Expected return binary stmt"),
                }
            }
            _ => panic!("Expected FuncDecl"),
        }
    }

    #[test]
    fn parses_function_call_statement() {
        let tokens = vec![
            t("foo"),
            Token::ParenOpen,
            t("x"),
            Token::Comma,
            t("y"),
            Token::ParenClose,
            Token::SemiColon,
        ];

        let ast = parse(tokens);

        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Stmt::FuncCall(call) => {
                assert_eq!(call.function_name, "foo");
                assert_eq!(call.arguments.len(), 2);
            }
            _ => panic!("Expected FuncCall"),
        }
    }

    #[test]
    fn parses_while_loop_with_assignment() {
        let tokens = vec![
            Token::While,
            Token::WhiteSpace,
            Token::ParenOpen,
            t("x"),
            Token::LessThan,
            t("10"),
            Token::ParenClose,
            Token::CurlyOpen,
            t("x"),
            Token::WhiteSpace,
            Token::Eq,
            Token::WhiteSpace,
            t("x"),
            Token::Sub,
            t("1"),
            Token::SemiColon,
            Token::CurlyClose,
        ];

        let mut it = tokens.iter().peekable();
        let (block, _) = parse_block(&mut it);

        assert_eq!(block.len(), 1);

        match &block[0] {
            Stmt::WhileStmt(while_stmt) => {
                assert_eq!(while_stmt.condition.lhs, "x");
                assert_eq!(while_stmt.condition.rhs.as_deref(), Some("10"));
                assert_eq!(while_stmt.block.len(), 1);
            }
            _ => panic!("Expected WhileStmt"),
        }
    }

    #[test]
    fn parses_if_statement() {
        let tokens = vec![
            Token::If,
            Token::WhiteSpace,
            Token::ParenOpen,
            t("x"),
            Token::DoubleEq,
            t("0"),
            Token::ParenClose,
            Token::CurlyOpen,
            Token::Return,
            Token::WhiteSpace,
            t("x"),
            Token::SemiColon,
            Token::CurlyClose,
        ];

        let mut it = tokens.iter().peekable();
        let (block, _) = parse_block(&mut it);

        assert_eq!(block.len(), 1);

        match &block[0] {
            Stmt::IfStmt(if_stmt) => {
                assert_eq!(if_stmt.condition.lhs, "x");
                assert_eq!(if_stmt.condition.rhs.as_deref(), Some("0"));
                assert!(if_stmt.if_block.is_empty());
                assert!(if_stmt.if_block_rt_val.is_some());
            }
            _ => panic!("Expected IfStmt"),
        }
    }

    #[test]
    fn parses_binary_expression() {
        let tokens = [t("a"), Token::Add, t("b")];

        let mut it = tokens.iter().peekable();
        let expr = parse_binary_stmt(&mut it, None);

        assert_eq!(expr.lhs, "a");
        assert_eq!(expr.rhs.as_deref(), Some("b"));
        assert_eq!(expr.op, Some(Token::Add));
    }
}
