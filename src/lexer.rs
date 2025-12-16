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

    if !temp_str.is_empty() {
        stream_token.push(get_literal_keyword(&temp_str));
    }

    stream_token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char_tokens() {
        let input = String::from("(){};=+-*%!,<> \n\t");
        let tokens = lex(input);

        let expected = vec![
            Token::ParenOpen,
            Token::ParenClose,
            Token::CurlyOpen,
            Token::CurlyClose,
            Token::SemiColon,
            Token::Eq,
            Token::Add,
            Token::Sub,
            Token::Mul,
            Token::Percent,
            Token::Exclaim,
            Token::Comma,
            Token::GreaterThan,
            Token::LessThan,
            Token::WhiteSpace,
            Token::NewLine,
            Token::Tab,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_keywords() {
        let input = String::from("function let return const if else while");
        let tokens = lex(input);

        let expected = vec![
            Token::Function,
            Token::WhiteSpace,
            Token::Let,
            Token::WhiteSpace,
            Token::Return,
            Token::WhiteSpace,
            Token::Const,
            Token::WhiteSpace,
            Token::If,
            Token::WhiteSpace,
            Token::Else,
            Token::WhiteSpace,
            Token::While,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_literal_identifiers() {
        let input = String::from("foo bar baz");
        let tokens = lex(input);

        let expected = vec![
            Token::Literal("foo".into()),
            Token::WhiteSpace,
            Token::Literal("bar".into()),
            Token::WhiteSpace,
            Token::Literal("baz".into()),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_mixed_expression() {
        let input = String::from("let x = a + b;");
        let tokens = lex(input);

        let expected = vec![
            Token::Let,
            Token::WhiteSpace,
            Token::Literal("x".into()),
            Token::WhiteSpace,
            Token::Eq,
            Token::WhiteSpace,
            Token::Literal("a".into()),
            Token::WhiteSpace,
            Token::Add,
            Token::WhiteSpace,
            Token::Literal("b".into()),
            Token::SemiColon,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment_literal() {
        let input = String::from("// comment");
        let tokens = lex(input);

        let expected = vec![
            Token::Comment,
            Token::WhiteSpace,
            Token::Literal("comment".into()),
        ];

        assert_eq!(tokens, expected);
    }
}
