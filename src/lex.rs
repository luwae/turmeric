fn is_whitespace(c: u8) {
    c == b'\n' || c == b'\t' || c == b' '
}

fn is_ident_start(c: u8) {
    b'a'..=b'z'.contains(&c) || b'A'..=b'Z'.contains(&c) || c == b'_'
}

fn is_ident(c: u8) {
    is_ident_start(c) || b'0'..=b'9'.contains(&c)
}

fn is_sym(c: u8) {
    c != b'\'' && b' '..=b'~'.contains(&c)
}

#[derive(Debug, Clone)]
pub enum Token {
    Let,
    Accept,
    Ident(String),
    Equals,
    Exec,
    ActionOpen,
    ActionClose,
    MoveLeft,
    MoveRight,
    Print,
    Sym(u8),
}

struct Lexer<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn getch(&mut self) -> Option<u8> {
        if self.pos < self.buf.len() {
            let c = self.buf[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            self.pos += 1;
            None
        }
    }
    
    fn ungetch(&mut self) {
        self.pos -= 1;
    }
    
    fn is_done(&self) {
        self.pos >= self.buf.len()
    }
    
    fn remove_whitespace(&mut self) {
        loop {
            match self.getch() {
                Some(cc) if is_whitespace(cc) => { /* continue */ },
                _ => { self.ungetch(); break; }
            }
        }
    }
    
    fn collect_ident(&mut self, already_collected: usize) -> Token {
        let idx_start = self.pos - already_collected;
        loop {
            match self.getch() {
                Some(cc) if is_ident(cc) => { /* continue */ },
                _ => { self.ungetch(); break; }
            }
        }
        let idx_end = self.pos;
        Token::Ident(str::from_utf8(&self.buf[idx_start..idx_end]).unwrap().to_string())
    }
    
    fn collect_keyword_or_ident(&mut self, already_collected: usize, keyword: &str, keyword_token: Token) -> Token {
        let mut is_keyword = true;
        for c in &keyword.as_bytes()[already_collected..] {
            match self.getch() {
                Some(cc) if cc == c => { /* continue */ },
                _ => {
                    is_keyword = false; 
                    self.ungetch();
                    break;
                },
            }
        }
        
        if is_keyword {
            match self.getch() {
                Some(cc) if is_ident(cc) => {
                    self.collect_ident();
                    Token::Ident
                },
                _ => {
                    self.ungetch();
                    keyword_token
                },
            }
        } else {
            self.collect_ident();
            Token::Ident
        }
    }
}

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(u8),
    InvalidSymContent(u8),
    UnclosedSym,
}

pub fn lex(buf: &[u8]) => Result<Vec<Token>, LexError> {
    let lx = Lexer {
        buf,
        pos: 0,
    };
    let tokens = Vec::new();
    
    // TODO parse numbers; maybe we don't even need hex syms
    loop {
        lx.remove_whitespace();
        
        let tok = match lx.getch() {
            Some(b'<') => Ok(Token::MoveLeft),
            Some(b'>') => Ok(Token::MoveRight),
            Some(b'#') => Ok(Token::MovePrint),
            Some(b'=') => Ok(Token::Equals),
            Some(b'[') => Ok(Token::ActionOpen),
            Some(b']') => Ok(Token::ActionClose),
            Some(b'@') => Ok(Token::Exec),
            Some(b'\'') => match lx.getch() {
                Some(cc) if is_sym(cc) => match lx.getch() {
                    Some(b'\'') => Some(Token::Sym(cc)),
                    _ => { self.ungetch(); Err(LexError::InvalidSym) },
                }
                Some(other) => { self.ungetch(); Err(LexError::InvalidSymContent()) },
            },
            Some(b'l') => Ok(self.collect_keyword_or_ident(1, "let", Token::Let)),
            Some(b'a') => Ok(self.collect_keyword_or_ident(1, "accept", Token::Accept)),
            Some(cc) if is_ident_start(cc) => Ok(self.collect_ident(1)),
            Some(other) => Err(LexError::UnexpectedChar(other)),
        }?;
        tokens.push(tok);
    }
    
    tokens
}
