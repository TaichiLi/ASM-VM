#![allow(dead_code)]
use crate::token::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

#[allow(non_camel_case_types)]
enum State {
    NONE,
    END_OF_FILE,
    IDENTIFIER,
    IMMEDIATE_DATA,
    DELIMITER,
}

pub struct Scanner {
    source_file_name_: String,
    file_: File,
    line_: i32,
    column_: i32,
    loc_: TokenLocation,
    current_char_: char,
    state_: State,
    token_: Token,
    buffer_: String,
    eof_flag_: bool,
    error_flag_: bool,
}

impl Scanner {
    pub fn new(source_file_name: String) -> Self {
        let file = match File::open(source_file_name.to_owned()) {
            Err(err) => panic!("When trying to open file {}, because {}, an error occurred.", err.to_string(),
                    &source_file_name),
            Ok(file) => file,
        };

        Scanner {
            source_file_name_: source_file_name.to_owned(),
            file_: file,
            line_: 1,
            column_: 0,
            loc_: TokenLocation::new(source_file_name, 1, 0),
            current_char_: Default::default(),
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

    fn make_token(&mut self, token_type: TokenType, token_value: TokenValue, loc: TokenLocation, name: String) {
        self.token_ = Token::new_token(token_type, token_value, loc, name);
        self.buffer_.clear();
        self.state_ = State::NONE;
    }

    fn make_int_token(&mut self, loc: TokenLocation, name: String, int_value: i32) {
        self.token_ = Token::new_int_token(loc, name, int_value);
        self.buffer_.clear();
        self.state_ = State::NONE;
    }

    fn get_next_char(&mut self) {
        let mut buffer = [0; 1];
        match self.file_.read_exact(&mut buffer) {
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

    fn get_peek_char(&mut self) -> char {
        let mut buffer = [0; 1];
        match self.file_.read_exact(&mut buffer) {
            Err(_e) => self.eof_flag_ = true,
            Ok(()) => buffer[0] = std::u8::MAX,
        };
        self.file_.seek(SeekFrom::Current(-1)).unwrap();
        buffer[0].into()
    }

    fn add_to_buffer(&mut self, ch: char) {
        self.buffer_.push(ch);
    }

    fn reduce_buffer(&mut self) {
        self.buffer_.pop();
    }

    fn error_token(&mut self, msg: &String) {
        eprintln!("{}", msg);
        self.error_flag_ = true;
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

    fn preprocess(&mut self) {
        loop {
            while self.current_char_.is_ascii_whitespace() && !self.eof_flag_ {
                self.get_next_char();
            }

            self.handle_comment();

            if !(self.current_char_.is_ascii_whitespace() || self.current_char_ == ';') || self.eof_flag_ {
                break;
            }
        }
    }

    /// Get the current token.
    ///
    /// # Examples
    ///
    /// ```
    /// let scanner = Scanner::new("/test.mjava");
    /// let token = scanner.get_token();
    /// ```
    pub fn get_token(&self) -> Token {
        self.token_.to_owned()
    }

    /// Get the next token.
    ///
    /// # Examples
    /// ```
    /// let scanner = Scanner::new("./test.mjava");
    /// let token = scanner.get_next_token();
    /// ```
    pub fn get_next_token(&mut self) -> Token {
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
                State::DELIMITER => self.handle_demiliter_state(),
            }

            match self.state_ {
                State::NONE => {
                    self.preprocess();

                    if self.eof_flag_ {
                        self.state_ = State::END_OF_FILE;
                    } else {
                        if self.current_char_.is_ascii_alphabetic() {
                            self.state_ = State::IDENTIFIER;
                        } else if self.current_char_.is_ascii_digit() {
                            self.state_ = State::IMMEDIATE_DATA;
                        } else {
                            self.state_ = State::DELIMITER;
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
            let int_value: i32 = match i32::from_str_radix(&self.buffer_.clone(), number_base) {
                Err(err) => {
                    self.error_report(&format!("When parse integer literal \"{}\", because {}, an error occurred.", self.buffer_,
                            err.to_string()));
                    self.buffer_.clear();
                    self.state_ = State::NONE;
                    std::i32::MAX
                },
                Ok(int_value) => int_value,
            };

            self.make_int_token(self.loc_.to_owned(), self.buffer_.to_owned(), int_value);
        }
    }
    
    fn handle_identifier_state(&mut self) {
        self.loc_ = self.get_token_location();

        self.add_to_buffer(self.current_char_);
        self.get_next_char();
        
        while self.current_char_.is_ascii_alphabetic() {
            self.add_to_buffer(self.current_char_);
            self.get_next_char();
        }
        
        let token_type;
        let token_value;
        
        match self.buffer_.as_str() {
            "mov" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::MOV;
            },
            "add" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::ADD;
            },
            "sub" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::SUB;
            },
            "mul" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::MUL;
            },
            "div" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::DIV;
            },
            "and" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::AND;
            },
            "or" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::OR;
            },
            "xor" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::XOR;
            },
            "not" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::NOT;
            },
            "neg" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::NEG;
            },
            "push" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::PUSH;
            },
            "pop" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::POP;
            },
            "cmp" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::CMP;
            },
            "je" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::JE;
            },
            "jne" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::JNE;
            },
            "jg" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::JG;
            },
            "jge" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::JGE;
            },
            "jl" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::JL;
            },
            "jle" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::JLE;
            },
            "call" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::CALL;
            },
            "ret" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::RET;
            },
            "int" => {
                token_type = TokenType::INSTRUCTION;
                token_value = TokenValue::INT;
            },
            "eax" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::EAX;
            },
            "ebx" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::EBX;
            },
            "ecx" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::ECX;
            },
            "edx" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::EDX;
            },
            "esi" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::ESI;
            },
            "edi" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::EDI;
            },
            "esp" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::ESP;
            },
            "ebp" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::EBP;
            },
            "eip" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::EIP;
            },
            "zf" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::ZF;
            },
            "sf" => {
                token_type = TokenType::REGISTER;
                token_value = TokenValue::SF;
            },
            _ => {
                token_type = TokenType::LABEL;
                token_value = TokenValue::LABEL;
            },
        }
        
        self.make_token(token_type, token_value, self.loc_.to_owned(), self.buffer_.to_owned());
    }

    fn handle_demiliter_state(&mut self) {
        self.loc_ = self.get_token_location();

        self.add_to_buffer(self.current_char_);

        let token_value = match self.buffer_.as_str() {
            "," => TokenValue::COMMA,
            "[" => TokenValue::LBRACK,
            "]" => TokenValue::RBRACK,
            ":" => TokenValue::COLON,
            _ => {
                self.error_report(&format!("Unknown demiliter: {}", &self.buffer_));
                TokenValue::UNKNOWN
            },
        };

        self.make_token(TokenType::DELIMITER, token_value, self.loc_.to_owned(), self.buffer_.to_owned());

        self.get_next_char();
    }
}
