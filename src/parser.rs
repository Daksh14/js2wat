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
