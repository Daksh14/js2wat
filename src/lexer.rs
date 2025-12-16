#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Function,
    ParenOpen,
    ParenClose,
    CurlyOpen,
    CurlyClose,
    SemiColon,
    Return,
    Comma,
    Tab,
    NewLine,
    // =
    Eq,
    // ==
    DoubleEq,
    // -
    Sub,
    // +
    Add,
    // *
    Mul,
    // %
    Percent,
    // !
    Exclaim,
    // <
    GreaterThan,
    // >
    LessThan,
    Let,
    Const,
    NotEq,
    While,
    If,
    Else,
    Comment,
    WhiteSpace,
    Literal(String),
    None,
}

fn get_byte_keyword(byte: &u8) -> Token {
    match byte {
        b'(' => Token::ParenOpen,
        b')' => Token::ParenClose,
        b'}' => Token::CurlyClose,
        b'{' => Token::CurlyOpen,
        b';' => Token::SemiColon,
        b'=' => Token::Eq,
        b'-' => Token::Sub,
        b'+' => Token::Add,
        b'%' => Token::Percent,
        b'*' => Token::Mul,
        b'!' => Token::Exclaim,
        b'<' => Token::GreaterThan,
        b'>' => Token::LessThan,
        b',' => Token::Comma,
        b' ' => Token::WhiteSpace,
        b'\n' => Token::NewLine,
        b'\t' => Token::Tab,
        _ => Token::None,
    }
}

fn get_literal_keyword(keyword: &str) -> Token {
    match keyword {
        "while" => Token::While,
        "let" => Token::Let,
        "return" => Token::Return,
        "function" => Token::Function,
        "const" => Token::Const,
        "if" => Token::If,
        "//" => Token::Comment,
        "else" => Token::Else,
        _ => Token::Literal(keyword.to_owned()),
    }
}

// Just a string for now
type TokenStream = String;

pub fn lex(stream: TokenStream) -> Vec<Token> {
    let mut stream_token = Vec::new();

    let mut temp_str = String::new();

    for byte in stream.as_bytes() {
        let lexed = get_byte_keyword(byte);

        match lexed {
            Token::None => {
                temp_str.push(*byte as char);
            }
            _ => {
                if !temp_str.is_empty() {
                    stream_token.push(get_literal_keyword(&temp_str));
                    temp_str.clear();
                }

                stream_token.push(lexed)
            }
        }
    }

    stream_token
}
