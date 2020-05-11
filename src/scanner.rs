use crate::token::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::collections::HashMap;

#[allow(non_camel_case_types)]
/// State of lexical analysis
enum State {
    NONE,
    END_OF_FILE,
    IDENTIFIER,
    IMMEDIATE_DATA,
    SYMBOL,
}

/// Lexical scanner
pub struct Scanner {
    source_file_name_: String,
    file_: Option<File>,
    line_: i32,
    column_: i32,
    loc_: TokenLocation,
    current_char_: char,
    dictionary_: HashMap<String, (TokenType, TokenValue)>,
    state_: State,
    token_: Token,
    buffer_: String,
    eof_flag_: bool,
    error_flag_: bool,
}

impl Default for Scanner {
    fn default() -> Self {
        Scanner {
            source_file_name_: Default::default(),
            file_: Default::default(),
            line_: 1,
            column_: 0,
            loc_: Default::default(),
            current_char_: Default::default(),
            dictionary_: Default::default(),
            state_: State::NONE,
            token_: Default::default(),
            buffer_: Default::default(),
            eof_flag_: false,
            error_flag_: false,
        }
    }
}

impl Scanner {
    /// New scanner from the name of source file.
    pub fn new(source_file_name: String) -> Self {
        let file = match File::open(source_file_name.to_owned()) {
            Err(err) => panic!("When trying to open file {}, because {}, an error occurred.", err.to_string(),
                    &source_file_name),
            Ok(file) => file,
        };

        let mut dictionary = HashMap::new();
        dictionary.insert("mov".to_string(), (TokenType::INSTRUCTION, TokenValue::MOV));
        dictionary.insert("movzx".to_string(), (TokenType::INSTRUCTION, TokenValue::MOVZX));
        dictionary.insert("movsx".to_string(), (TokenType::INSTRUCTION, TokenValue::MOVSX));
        dictionary.insert("add".to_string(), (TokenType::INSTRUCTION, TokenValue::ADD));
        dictionary.insert("sub".to_string(), (TokenType::INSTRUCTION, TokenValue::SUB));
        dictionary.insert("inc".to_string(), (TokenType::INSTRUCTION, TokenValue::INC));
        dictionary.insert("dec".to_string(), (TokenType::INSTRUCTION, TokenValue::DEC));
        dictionary.insert("mul".to_string(), (TokenType::INSTRUCTION, TokenValue::MUL));
        dictionary.insert("imul".to_string(), (TokenType::INSTRUCTION, TokenValue::IMUL));
        dictionary.insert("div".to_string(), (TokenType::INSTRUCTION, TokenValue::DIV));
        dictionary.insert("idiv".to_string(), (TokenType::INSTRUCTION, TokenValue::IDIV));
        dictionary.insert("and".to_string(), (TokenType::INSTRUCTION, TokenValue::AND));
        dictionary.insert("or".to_string(), (TokenType::INSTRUCTION, TokenValue::OR));
        dictionary.insert("xor".to_string(), (TokenType::INSTRUCTION, TokenValue::XOR));
        dictionary.insert("not".to_string(), (TokenType::INSTRUCTION, TokenValue::NOT));
        dictionary.insert("neg".to_string(), (TokenType::INSTRUCTION, TokenValue::NEG));
        dictionary.insert("push".to_string(), (TokenType::INSTRUCTION, TokenValue::PUSH));
        dictionary.insert("pop".to_string(), (TokenType::INSTRUCTION, TokenValue::POP));
        dictionary.insert("shl".to_string(), (TokenType::INSTRUCTION, TokenValue::SHL));
        dictionary.insert("sal".to_string(), (TokenType::INSTRUCTION, TokenValue::SHL));
        dictionary.insert("shr".to_string(), (TokenType::INSTRUCTION, TokenValue::SHR));
        dictionary.insert("sar".to_string(), (TokenType::INSTRUCTION, TokenValue::SAR));
        dictionary.insert("cmp".to_string(), (TokenType::INSTRUCTION, TokenValue::CMP));
        dictionary.insert("jmp".to_string(), (TokenType::INSTRUCTION, TokenValue::JMP));
        dictionary.insert("je".to_string(), (TokenType::INSTRUCTION, TokenValue::JE));
        dictionary.insert("jz".to_string(), (TokenType::INSTRUCTION, TokenValue::JE));
        dictionary.insert("jne".to_string(), (TokenType::INSTRUCTION, TokenValue::JNE));
        dictionary.insert("jnz".to_string(), (TokenType::INSTRUCTION, TokenValue::JNE));
        dictionary.insert("jg".to_string(), (TokenType::INSTRUCTION, TokenValue::JG));
        dictionary.insert("jnle".to_string(), (TokenType::INSTRUCTION, TokenValue::JG));
        dictionary.insert("jge".to_string(), (TokenType::INSTRUCTION, TokenValue::JGE));
        dictionary.insert("jnl".to_string(), (TokenType::INSTRUCTION, TokenValue::JGE));
        dictionary.insert("jl".to_string(), (TokenType::INSTRUCTION, TokenValue::JL));
        dictionary.insert("jnge".to_string(), (TokenType::INSTRUCTION, TokenValue::JL));
        dictionary.insert("jle".to_string(), (TokenType::INSTRUCTION, TokenValue::JLE));
        dictionary.insert("jng".to_string(), (TokenType::INSTRUCTION, TokenValue::JLE));
        dictionary.insert("ja".to_string(), (TokenType::INSTRUCTION, TokenValue::JA));
        dictionary.insert("jnbe".to_string(), (TokenType::INSTRUCTION, TokenValue::JA));
        dictionary.insert("jae".to_string(), (TokenType::INSTRUCTION, TokenValue::JAE));
        dictionary.insert("jnb".to_string(), (TokenType::INSTRUCTION, TokenValue::JAE));
        dictionary.insert("jb".to_string(), (TokenType::INSTRUCTION, TokenValue::JB));
        dictionary.insert("jnae".to_string(), (TokenType::INSTRUCTION, TokenValue::JB));
        dictionary.insert("jbe".to_string(), (TokenType::INSTRUCTION, TokenValue::JBE));
        dictionary.insert("jna".to_string(), (TokenType::INSTRUCTION, TokenValue::JBE));
        dictionary.insert("call".to_string(), (TokenType::INSTRUCTION, TokenValue::CALL));
        dictionary.insert("ret".to_string(), (TokenType::INSTRUCTION, TokenValue::RET));
        dictionary.insert("enter".to_string(), (TokenType::INSTRUCTION, TokenValue::ENTER));
        dictionary.insert("leave".to_string(), (TokenType::INSTRUCTION, TokenValue::LEAVE));
        dictionary.insert("eax".to_string(), (TokenType::REGISTER, TokenValue::EAX));
        dictionary.insert("ax".to_string(), (TokenType::REGISTER, TokenValue::AX));
        dictionary.insert("ah".to_string(), (TokenType::REGISTER, TokenValue::AH));
        dictionary.insert("al".to_string(), (TokenType::REGISTER, TokenValue::AL));
        dictionary.insert("ebx".to_string(), (TokenType::REGISTER, TokenValue::EBX));
        dictionary.insert("bx".to_string(), (TokenType::REGISTER, TokenValue::BX));
        dictionary.insert("bh".to_string(), (TokenType::REGISTER, TokenValue::BH));
        dictionary.insert("bl".to_string(), (TokenType::REGISTER, TokenValue::BL));
        dictionary.insert("ecx".to_string(), (TokenType::REGISTER, TokenValue::ECX));
        dictionary.insert("cx".to_string(), (TokenType::REGISTER, TokenValue::CX));
        dictionary.insert("ch".to_string(), (TokenType::REGISTER, TokenValue::CH));
        dictionary.insert("cl".to_string(), (TokenType::REGISTER, TokenValue::CL));
        dictionary.insert("edx".to_string(), (TokenType::REGISTER, TokenValue::EDX));
        dictionary.insert("dx".to_string(), (TokenType::REGISTER, TokenValue::DX));
        dictionary.insert("dh".to_string(), (TokenType::REGISTER, TokenValue::DH));
        dictionary.insert("dl".to_string(), (TokenType::REGISTER, TokenValue::DL));
        dictionary.insert("esi".to_string(), (TokenType::REGISTER, TokenValue::ESI));
        dictionary.insert("si".to_string(), (TokenType::REGISTER, TokenValue::SI));
        dictionary.insert("edi".to_string(), (TokenType::REGISTER, TokenValue::EDI));
        dictionary.insert("di".to_string(), (TokenType::REGISTER, TokenValue::DI));
        dictionary.insert("esp".to_string(), (TokenType::REGISTER, TokenValue::ESP));
        dictionary.insert("sp".to_string(), (TokenType::REGISTER, TokenValue::SP));
        dictionary.insert("ebp".to_string(), (TokenType::REGISTER, TokenValue::EBP));
        dictionary.insert("bp".to_string(), (TokenType::REGISTER, TokenValue::BP));
        dictionary.insert("ptr".to_string(), (TokenType::KEYWORD, TokenValue::PTR));
        dictionary.insert("byte".to_string(), (TokenType::KEYWORD, TokenValue::BYTE));
        dictionary.insert("word".to_string(), (TokenType::KEYWORD, TokenValue::WORD));
        dictionary.insert("dword".to_string(), (TokenType::KEYWORD, TokenValue::DWORD));

        Scanner {
            source_file_name_: source_file_name.to_owned(),
            file_: Some(file),
            line_: 1,
            column_: 0,
            loc_: TokenLocation::new(source_file_name, 1, 0),
            current_char_: Default::default(),
            dictionary_: dictionary,
            state_: State::NONE,
            token_: Default::default(),
            buffer_: Default::default(),
            eof_flag_: false,
            error_flag_: false,
        }
    }

    fn get_token_location(&self) -> TokenLocation {
        TokenLocation::new(self.source_file_name_.to_owned(), self.line_, self.column_)
    }

    /// Make a `instruction`, `register` or `label` token and reset scanner.
    fn make_token(&mut self, token_type: TokenType, token_value: TokenValue, loc: TokenLocation, name: String) {
        self.token_ = Token::new_token(token_type, token_value, loc, name);
        self.buffer_.clear();
        self.state_ = State::NONE;
    }

    /// Make a `immediate data` token and reset scanner.
    fn make_int_token(&mut self, loc: TokenLocation, name: String, int_value: u32) {
        self.token_ = Token::new_int_token(loc, name, int_value);
        self.buffer_.clear();
        self.state_ = State::NONE;
    }

    /// Make a `symbol` token and reset scanner.
    fn make_symbol_token(&mut self, token_value: TokenValue, loc: TokenLocation, name: String, int_value: i32) {
        self.token_ = Token::new_symbol_token(token_value, loc, name, int_value);
        self.buffer_.clear();
        self.state_ = State::NONE;
    }

    /// Get one char from source file and advance the sequence.
    fn get_next_char(&mut self) {
        let mut buffer = [0; 1];
        match self.file_.as_ref().unwrap().read_exact(&mut buffer) {
            Err(_e) => {
                self.eof_flag_ = true;
                self.current_char_ = std::char::MAX;
            },
            Ok(()) => self.current_char_ = buffer[0].into(),
        }

        if self.current_char_ == '\n' {
            self.line_ = self.line_ + 1;
            self.column_ = 0;
        } else {
            self.column_ = self.column_ + 1;
        }
    }

    /// Get one char from source file without advancing the sequence.
    fn get_peek_char(&mut self) -> char {
        let mut buffer = [0; 1];
        match self.file_.as_ref().unwrap().read_exact(&mut buffer) {
            Err(_e) => self.eof_flag_ = true,
            Ok(()) => buffer[0] = std::u8::MAX,
        };
        self.file_.as_ref().unwrap().seek(SeekFrom::Current(-1)).unwrap();
        buffer[0].into()
    }

    /// Add current char to buffer.
    fn add_to_buffer(&mut self, ch: char) {
        self.buffer_.push(ch);
    }

    fn error_token(&mut self, msg: &String) {
        self.error_flag_ = true;
        panic!("{}", msg);
    }

    fn error_report(&mut self, msg: &String) {
        self.error_token(&format!("Token Error: {}{}", self.get_token_location().to_string(), msg));
    }

    fn handle_comment(&mut self) {
        self.loc_ = self.get_token_location();

        if self.current_char_ == ';' {
            self.get_next_char();

            while self.current_char_ != '\n' && !self.eof_flag_ {
                self.get_next_char();
            }

            if !self.eof_flag_ {
                self.get_next_char();
            }
        }
    }

    fn handle_directive(&mut self) {
        self.loc_ = self.get_token_location();

        if self.current_char_ == '.' {
            self.get_next_char();

            while self.current_char_ != '\n' && !self.eof_flag_ {
                self.get_next_char();
            }

            if !self.eof_flag_ {
                self.get_next_char();
            }
        }
    }

    fn preprocess(&mut self) {
        loop {
            while self.current_char_.is_ascii_whitespace() && !self.eof_flag_ {
                self.get_next_char();
            }

            self.handle_directive();
            self.handle_comment();

            if !(self.current_char_.is_ascii_whitespace() || self.current_char_ == ';' || self.current_char_ == '.') || self.eof_flag_ {
                break;
            }
        }
    }

    /// Get the current token.
    ///
    /// # Examples
    ///
    /// ```
    /// let scanner = Scanner::new("/test.asm");
    /// let token = scanner.get_token();
    /// ```
    pub fn get_token(&self) -> Token {
        if self.file_.is_some() {
            self.token_.to_owned()
        } else {
            panic!("Source File has not been set!");
        }
    }

    /// Get the next token.
    ///
    /// # Examples
    /// ```
    /// let scanner = Scanner::new("./test.asm");
    /// let token = scanner.get_next_token();
    /// ```
    pub fn get_next_token(&mut self) -> Token {
        if self.file_.is_none() {
            panic!("Source file has not been set!");
        }

        let mut matched;

        loop {
            self.error_flag_ = false;

            match self.state_ {
                State::NONE => matched = false,
                _ => matched = true,
            }

            match self.state_ {
                State::NONE => self.get_next_char(),
                State::END_OF_FILE => self.handle_eof_state(),
                State::IDENTIFIER => self.handle_identifier_state(),
                State::IMMEDIATE_DATA => self.handle_immedidate_data_state(),
                State::SYMBOL => self.handle_symbol_state(),
            }

            match self.state_ {
                State::NONE => {
                    self.preprocess();

                    if self.eof_flag_ {
                        self.state_ = State::END_OF_FILE;
                    } else {
                        if self.current_char_.is_ascii_alphabetic() || self.current_char_ == '_' {
                            self.state_ = State::IDENTIFIER;
                        } else if self.current_char_.is_ascii_digit() {
                            self.state_ = State::IMMEDIATE_DATA;
                        } else {
                            self.state_ = State::SYMBOL;
                        }
                    }
                },
                _ => {},
            }

            if matched && !self.error_flag_ {
                break;
            }
        }

        self.token_.to_owned()
    }

    fn handle_eof_state(&mut self) {
        self.loc_ = self.get_token_location();
        self.make_token(TokenType::END_OF_FILE, TokenValue::END_OF_FILE, self.loc_.to_owned(), "END_OF_FILE".to_string());
    }

    fn handle_digit(&mut self) {
        self.add_to_buffer(self.current_char_);
        self.get_next_char();

        while self.current_char_.is_ascii_digit() {
            self.add_to_buffer(self.current_char_);
            self.get_next_char();
        }
    }

    fn handle_xdigit(&mut self) {
        let mut read_flag = false;

        while self.current_char_.is_ascii_hexdigit() {
            read_flag = true;
            self.add_to_buffer(self.current_char_);
            self.get_next_char();
        }

        if !read_flag {
            self.error_report(&"Hexadecimal number format error.".to_string());
        }
    }

    fn handle_odigit(&mut self) {
        let mut read_flag = false;

        while self.current_char_ >= '0' && self.current_char_ <= '7' {
            read_flag = true;
            self.add_to_buffer(self.current_char_);
            self.get_next_char();
        }

        if !read_flag
        {
            self.error_report(&"Octal number format error.".to_string());
        }
    }

    fn handle_immedidate_data_state(&mut self) {
        self.loc_ = self.get_token_location();

        let mut number_base = 10;

        if self.current_char_ == '0' && (self.get_peek_char() == 'x' || self.get_peek_char() == 'X') {
            number_base = 16;

            self.get_next_char();
            self.get_next_char();
        }

        if self.current_char_ == '0' && self.get_peek_char() >= '0' && self.get_peek_char() <= '7' {
            number_base = 8;

            self.get_next_char();
        }

        match number_base {
            10 => self.handle_digit(),
            16 => self.handle_xdigit(),
            8 => self.handle_odigit(),
            _ => {},
        }

        if !self.error_flag_ {
            let int_value: u32 = match u32::from_str_radix(&self.buffer_.clone(), number_base) {
                Err(err) => {
                    self.error_report(&format!("When parse integer literal \"{}\", because {}, an error occurred.", self.buffer_,
                            err.to_string()));
                    self.buffer_.clear();
                    self.state_ = State::NONE;
                    std::u32::MAX
                },
                Ok(int_value) => int_value,
            };

            self.make_int_token(self.loc_.to_owned(), self.buffer_.to_owned(), int_value);
        }
    }

    /// handle `instruction`, `register` and `label`.
    fn handle_identifier_state(&mut self) {
        self.loc_ = self.get_token_location();

        self.add_to_buffer(self.current_char_);
        self.get_next_char();

        while self.current_char_.is_ascii_alphanumeric() || self.current_char_ == '_'{
            self.add_to_buffer(self.current_char_);
            self.get_next_char();
        }

        let (token_type, token_value) = match self.dictionary_.get(&self.buffer_.to_lowercase()) {
            Some(info) => *info,
            None => (TokenType::LABEL, TokenValue::LABEL),
        };        

        self.make_token(token_type, token_value, self.loc_.to_owned(), self.buffer_.to_owned());
    }

    fn handle_symbol_state(&mut self) {
        self.loc_ = self.get_token_location();

        self.add_to_buffer(self.current_char_);

        let (token_value, precedence) =  match self.buffer_.as_str() {
            "+" => (TokenValue::PLUS, 10),
            "-" => (TokenValue::MINUS, 10),
            "*" => (TokenValue::TIMES, 20),
            "," => (TokenValue::COMMA, -1),
            "[" => (TokenValue::LBRACK, -1),
            "]" => (TokenValue::RBRACK, -1),
            ":" => (TokenValue::COLON, -1),
            _ => {
                self.error_report(&format!("Unknown symbol: {}", &self.buffer_));
                (TokenValue::UNKNOWN, -1)
            },
        };

        self.make_symbol_token(token_value, self.loc_.to_owned(), self.buffer_.to_owned(), precedence);

        self.get_next_char();
    }
}
