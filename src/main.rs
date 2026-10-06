//! # Prototype used for testing
//! **TODO:** Initialize Cargo project and seperate code into modules.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    StringLiteral(String),
    Number(f64),
    Arrow,      // ->
    Let,        // let
    Equal,      // =
    In,         // in
    OpenBrace,  // {
    CloseBrace, // }
    Comma,      // ,
}

#[derive(Debug, Clone)]
enum Expr {
    Variable(String),
    StringLit(String),
    NumLit(f64),
    // x -> x
    Function { arg: String, body: Box<Expr> },
    // let x = y in z
    LetIn { var: String, val: Box<Expr>, body: Box<Expr> },
    // A Map is a list of functions (patterns -> expressions)
    Map(Vec<Expr>),
    // Function call / Map lookup: (func, arg)
    Call { func: Box<Expr>, arg: Box<Expr> },
}

fn lexer(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => { chars.next(); }
            '#' => {
                // Comment: skip to end of line
                while let Some(&c) = chars.peek() {
                    if c == '\n' { break; }
                    chars.next();
                }
            }
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    panic!("Unexpected character '-' without '>'");
                }
            }
            '=' => { chars.next(); tokens.push(Token::Equal); }
            '{' => { chars.next(); tokens.push(Token::OpenBrace); }
            '}' => { chars.next(); tokens.push(Token::CloseBrace); }
            ',' => { chars.next(); tokens.push(Token::Comma); }
            '"' => {
                chars.next(); // Consume opening quote
                let mut string = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' { chars.next(); break; }
                    string.push(chars.next().unwrap());
                }
                tokens.push(Token::StringLiteral(string));
            }
            c if c.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(chars.next().unwrap());
                    } else { break; }
                }
                tokens.push(Token::Number(num_str.parse().unwrap()));
            }
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(chars.next().unwrap());
                    } else { break; }
                }
                match ident.as_str() {
                    "let" => tokens.push(Token::Let),
                    "in" => tokens.push(Token::In),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            _ => panic!("Unexpected character: {}", ch),
        }
    }
    tokens
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else { None }
    }

    // Parses overall expressions, including function definitions and let-ins
    fn parse_expr(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        // Right after an expression, check for an arrow -> (Function Definition)
        if let Some(Token::Arrow) = self.peek() {
            self.advance(); // consume ->
            if let Expr::StringLit(arg) = expr {
                let body = self.parse_expr();
                return Expr::Function { arg, body: Box::new(body) };
            } else {
                panic!("Left side of '->' must be an identifier pattern");
            }
        }

        // Left-associative function application: "f x y" -> Call(Call(f, x), y)
        while let Some(next_tok) = self.peek() {
            // If the next token safely looks like the start of a new argument expression...
            match next_tok {
                Token::Identifier(_) | Token::StringLiteral(_) | Token::Number(_) | Token::Let | Token::OpenBrace => {
                    let arg = self.parse_primary();
                    expr = Expr::Call { func: Box::new(expr), arg: Box::new(arg) };
                }
                _ => break, // Comma, in, braces, etc terminate an argument stream
            }
        }

        expr
    }

    // Parses basic atomic structures
    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Some(Token::Identifier(name)) => Expr::Variable(name),
            Some(Token::StringLiteral(s)) => Expr::StringLit(s),
            Some(Token::Number(n)) => Expr::NumLit(n),
            Some(Token::Let) => {
                let var = match self.advance() {
                    Some(Token::Identifier(v)) => v,
                    _ => panic!("Expected identifier after 'let'"),
                };
                assert_eq!(self.advance(), Some(Token::Equal), "Expected '=' after let identifier");
                let val = self.parse_expr();
                assert_eq!(self.advance(), Some(Token::In), "Expected 'in' after let value");
                let body = self.parse_expr();
                Expr::LetIn { var, val: Box::new(val), body: Box::new(body) }
            }
            Some(Token::OpenBrace) => {
                let mut mappings = Vec::new();
                while let Some(tok) = self.peek() {
                    if tok == &Token::CloseBrace { break; }
                    mappings.push(self.parse_expr());
                    if self.peek() == Some(&Token::Comma) { self.advance(); }
                }
                assert_eq!(self.advance(), Some(Token::CloseBrace), "Expected '}}' to close map");
                Expr::Map(mappings)
            }
            _ => panic!("Unexpected token sequence in primary expression"),
        }
    }
}

fn main() {
    let source_code = r#"
        let obj = {
            "status" -> "success",
            "nested" -> {
                "id" -> 1234
            }
        } in obj "nested" "id"
    "#;

    let tokens = lexer(source_code);
    println!("--- Tokens --- \n{:?}\n", tokens);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expr();
    println!("--- Abstract Syntax Tree --- \n{:#?}", ast);
}

// Expected output:
/*
--- Tokens --- 
[Let, Identifier("obj"), Equal, OpenBrace, StringLiteral("status"), Arrow, StringLiteral("success"), Comma, StringLiteral("nested"), Arrow, OpenBrace, StringLiteral("id"), Arrow, Number(1234.0), CloseBrace, CloseBrace, In, Identifier("obj"), StringLiteral("nested"), StringLiteral("id")]

--- Abstract Syntax Tree --- 
LetIn {
    var: "obj",
    val: Map(
        [
            Function {
                arg: "status",
                body: StringLit(
                    "success",
                ),
            },
            Function {
                arg: "nested",
                body: Map(
                    [
                        Function {
                            arg: "id",
                            body: NumLit(
                                1234.0,
                            ),
                        },
                    ],
                ),
            },
        ],
    ),
    body: Call {
        func: Call {
            func: Variable(
                "obj",
            ),
            arg: StringLit(
                "nested",
            ),
        },
        arg: StringLit(
            "id",
        ),
    },
}
*/
