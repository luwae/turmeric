fn is_whitespace(c: u8) -> bool {
    c == b'\n' || c == b'\t' || c == b' '
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

fn is_sym(c: u8) -> bool {
    c != b'\'' && (b' '..=b'~').contains(&c)
}

#[derive(Debug, Clone)]
pub enum Token {
    Let,
    Accept,
    Reject,
    Ident(String),
    Equals,
    Exec,
    ActionOpen,
    ActionClose,
    ParensOpen,
    ParensClose,
    BracesOpen,
    BracesClose,
    Bar,
    MoveLeft,
    MoveRight,
    Print,
    Sym(u8),
}

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(u8),
    InvalidSymContent(u8),
    UnclosedSym,
    SymNumberTooBig,
}

struct Lexer<'a> {
    buf: &'a [u8],
    pos: usize,
    lineno: usize,
}

impl<'a> Lexer<'a> {
    fn getch(&mut self) -> Option<u8> {
        if self.pos < self.buf.len() {
            let c = self.buf[self.pos];
            if c == b'\n' {
                self.lineno += 1;
            }
            self.pos += 1;
            Some(c)
        } else {
            self.pos += 1;
            None
        }
    }
    
    fn ungetch(&mut self) {
        self.pos -= 1;
        if self.pos < self.buf.len() && self.buf[self.pos] == b'\n' {
            self.lineno -= 1;
        }
    }
    
    fn remove_whitespace(&mut self) {
        loop {
            match self.getch() {
                Some(cc) if is_whitespace(cc) => { /* continue */ },
                _ => { self.ungetch(); break; }
            }
        }
    }
    
    fn collect_ident(&mut self) -> Token {
        let idx_start = self.pos - 1;
        loop {
            match self.getch() {
                Some(cc) if is_ident(cc) => { /* continue */ },
                _ => { self.ungetch(); break; }
            }
        }
        let idx_end = self.pos;
        Token::Ident(std::str::from_utf8(&self.buf[idx_start..idx_end]).unwrap().to_string())
    }
    
    fn collect_num(&mut self, first_digit: u8) -> Result<Token, LexError> {
        let mut num: usize = (first_digit - b'0') as usize;
        loop {
            match self.getch() {
                Some(cc) if cc.is_ascii_digit() => {
                    num = num * 10 + (cc - b'0') as usize;
                    if num > 255 {
                        return Err(LexError::SymNumberTooBig);
                    }
                }
                _ => {
                    self.ungetch();
                    return Ok(Token::Sym(num as u8));
                }
            }
        }
    }

    fn collect_sym(&mut self) -> Result<Token, LexError> {
        match self.getch() {
            Some(cc) if is_sym(cc) => match self.getch() {
                Some(b'\'') => Ok(Token::Sym(cc)),
                _ => {
                    self.ungetch();
                    Err(LexError::UnclosedSym)
                },
            },
            Some(other) => {
                self.ungetch();
                Err(LexError::InvalidSymContent(other))
            },
            _ => {
                self.ungetch();
                Err(LexError::UnclosedSym)
            }
        }
    }
}

fn replace_keywords(tokens: &mut Vec<Token>) {
    for tok in tokens {
        let maybe_keyword = match tok {
            Token::Ident(s) => match s.as_str() {
                "let" => Some(Token::Let),
                "accept" => Some(Token::Accept),
                "reject" => Some(Token::Reject),
                _ => None,
            },
            _ => None,
        };
        if let Some(new_tok) = maybe_keyword {
            *tok = new_tok;
        }
    }
}

pub fn lex(buf: &[u8]) -> Result<Vec<Token>, LexError> {
    let mut lx = Lexer {
        buf,
        pos: 0,
        lineno: 1,
    };
    let mut tokens = Vec::new();
    
    loop {
        lx.remove_whitespace();
        let c = match lx.getch() {
            Some(cc) => cc,
            None => break,
        };

        let tok = match c {
            b'<' => Ok(Token::MoveLeft),
            b'>' => Ok(Token::MoveRight),
            b'#' => Ok(Token::Print),
            b'=' => Ok(Token::Equals),
            b'[' => Ok(Token::ActionOpen),
            b']' => Ok(Token::ActionClose),
            b'(' => Ok(Token::ParensOpen),
            b')' => Ok(Token::ParensClose),
            b'{' => Ok(Token::BracesOpen),
            b'}' => Ok(Token::BracesClose),
            b'|' => Ok(Token::Bar),
            b'@' => Ok(Token::Exec),
            b'\'' => lx.collect_sym(),
            cc if cc.is_ascii_digit() => lx.collect_num(cc),
            cc if is_ident_start(cc) => Ok(lx.collect_ident()),
            other => Err(LexError::UnexpectedChar(other)),
        }?;
        tokens.push(tok);
    }
    
    replace_keywords(&mut tokens);
    Ok(tokens)
}
