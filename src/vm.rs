use crate::token::*;
use crate::scanner::*;
use std::collections::HashMap;
use std::vec::Vec;
use std::result::Result;
use std::convert::TryInto;

const MAX: usize = 1024 * 1024;
// const BYTE: u32= 0b1111_1111_1111_1111_1111_1111_0000_0000;
// const WORD: u32 = 0b1111_1111_1111_1111_0000_0000_0000_0000;

/// Visual Machine for x86 assembly
pub struct VM {
    /// simulate the `stack`
    stack: [u8; MAX],
    /// simulate the `memory`
    memory: [u8; MAX],
    /// simulate the `text`
    text: Vec<Token>,
    /// label location table, to implement `call` instruction.
    index: HashMap<String, i32>,
    /// `eax`, accumulator register
    eax: [u8; 4],
    /// `ebx`, base register
    ebx: [u8; 4],
    /// `ecx`, counter register
    ecx: [u8; 4],
    /// `edx`, data register
    edx: [u8; 4],
    /// `esi`, source index register
    esi: [u8; 4],
    /// `edi`, destination index register
    edi: [u8; 4],
    /// `esp`, stack pointer register
    esp: [u8; 4],
    /// `ebp`, base pointer register
    ebp: [u8; 4],
    /// `eip`, instruction pointer register
    eip: [u8; 4],
    /// `cf`, carry flag
    cf: bool,
    /// `zf`, zero flag
    zf: bool,
    /// `sf`, sign flag
    sf: bool,
    /// `of`, overflow flag
    of: bool,
    /// lexical scanner
    scanner: Scanner,
    /// call stack depth
    depth: u8,
    /// error flag
    error_flag_: bool,
}

#[allow(dead_code)]
impl VM {
    /// New VM from a assembly source file.
    pub fn new(source_file_name: String) -> Self {
        VM {
            stack: [0; MAX],
            memory: [0; MAX],
            text: Vec::new(),
            index: HashMap::new(),
            eax: [0; 4],
            ebx: [0; 4],
            ecx: [0; 4],
            edx: [0; 4],
            esi: [0; 4],
            edi: [0; 4],
            esp: ((MAX - 1) as u32).to_le_bytes(),
            ebp: ((MAX - 1) as u32).to_le_bytes(),
            eip: [0; 4],
            cf: false,
            zf: false,
            sf: false,
            of: false,
            scanner: Scanner::new(source_file_name),
            depth: 1,
            error_flag_: false,
        }
    }

    fn error_syntax(&mut self, msg: &String) {
        self.error_flag_ = true;
        panic!("{}", msg);
    }

    fn error_report(&mut self, msg: &String) {
        self.error_syntax(&format!("Syntax Error: {} {}", self.text[self.get_eip()].get_token_location().to_string(),
                    msg));
    }

    fn expect_token_type(&mut self, token_type: TokenType, token_name: String, advance_to_next_token: bool) -> bool {
        if self.text[self.get_eip()].get_token_type() != token_type {
            self.error_report(&format!("Expected \"{}\", but find \"{}\"", token_name,
                        self.text[self.get_eip()].get_token_name()));
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn expect_token_value(&mut self, token_value: TokenValue, token_name: String, advance_to_next_token: bool) -> bool {
        if self.text[self.get_eip()].get_token_value() != token_value {
            self.error_report(&format!("Expected \"{}\", but find \"{}\"", token_name,
                        self.text[self.get_eip()].get_token_name()));
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn validate_token_type(&mut self, token_type: TokenType, advance_to_next_token: bool) -> bool {
        if self.text[self.get_eip()].get_token_type() != token_type {
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn validate_token_value(&mut self, token_value: TokenValue, advance_to_next_token: bool) -> bool {
        if self.text[self.get_eip()].get_token_value() != token_value {
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn get_eip(&self) -> usize {
        u32::from_le_bytes(self.eip) as usize
    }

    /// change `eip`.
    ///
    /// eip += displacement;
    fn go_from_here(&mut self, displacement: i32) {
        let value: u32 = match (self.get_eip() as i32 + displacement).try_into() {
            Ok(value) => value,
            Err(err) => panic!("Invaild memory address: {}", err),
        };

        self.eip = value.to_le_bytes();
    }

    /// Preprocess assembly source code.
    ///
    /// 1. Read all token from source file, and store into `self.text`.
    /// 2. Record the location of `label`, and store into `self.index`.
    /// 3. Replace the the `label` in `call label` instruction with the corresponding displacement.
    fn preprocess(&mut self) {
        let mut count = -1;
        let mut entrance = 0;

        loop {
            let last_token = self.scanner.get_token();

            self.scanner.get_next_token();
            count = count + 1;

            let token = self.scanner.get_token();

            if token.get_token_value() == TokenValue::COLON {
                if last_token.get_token_type() != TokenType::LABEL {
                    panic!("Syntax Error: {} Expected \"label\", but find \"{}\"",
                            token.get_token_location().to_string(), token.get_token_name());
                }

                self.index.insert(last_token.get_token_name(), count - 1);

                match last_token.get_token_name().as_str() {
                    "main" | "start" | "_main" | "_start" => entrance = count - 1,
                    _ => {},
                }
            }

            match token.get_token_type() {
                TokenType::END_OF_FILE => break,
                _ => self.text.push(token),
            }
        }

        let mut flag = false;
        count = -1;

        for token in &mut self.text {
            count = count + 1;

            if !flag {
                match token.get_token_value() {
                    TokenValue::CALL | TokenValue::JMP | TokenValue::JE | TokenValue::JNE | TokenValue::JG | TokenValue::JGE |
                        TokenValue::JL | TokenValue::JLE | TokenValue::JA | TokenValue::JAE | TokenValue::JB |
                        TokenValue::JBE => {
                            flag = true;
                    },
                    _ => {},
                }
            } else {
                if token.get_token_type() != TokenType::LABEL {
                    panic!("Syntax Error: {} Expected \"label\", but find \"{}\"",
                            token.get_token_location().to_string(), token.get_token_name());
                }

                let label_name = token.get_token_name();

                if !self.index.contains_key(&label_name) {
                    panic!("Syntax Error: {} Unknown label: \"{}\"", token.get_token_location().to_string(), label_name);
                }

                let label_address = self.index.get(&label_name).unwrap();

                token.set_token_type(TokenType::IMMEDIATE_DATA);
                token.set_int_value(label_address - count - 1);

                flag = false;
            }
        }

        self.eip = (entrance as u32).to_le_bytes();
    }

    fn parse_register(&mut self) -> Result<(*mut [u8], usize, usize), String> {
        self.go_from_here(1);

        match self.text[self.get_eip() - 1].get_token_value() {
            TokenValue::EAX => return Ok((&mut self.eax as *mut [u8], 0, 4)),
            TokenValue::AX => return Ok((&mut self.eax as *mut [u8], 0, 2)),
            TokenValue::AH => return Ok((&mut self.eax as *mut [u8], 1, 1)),
            TokenValue::AL => return Ok((&mut self.eax as *mut [u8], 0, 1)),
            TokenValue::EBX => return Ok((&mut self.ebx as *mut [u8], 0, 4)),
            TokenValue::BX => return Ok((&mut self.ebx as *mut [u8], 0, 2)),
            TokenValue::BH => return Ok((&mut self.ebx as *mut [u8], 1, 1)),
            TokenValue::BL => return Ok((&mut self.ebx as *mut [u8], 0, 1)),
            TokenValue::ECX => return Ok((&mut self.ecx as *mut [u8], 0, 4)),
            TokenValue::CX => return Ok((&mut self.ecx as *mut [u8], 0, 2)),
            TokenValue::CH => return Ok((&mut self.ecx as *mut [u8], 1, 1)),
            TokenValue::CL => return Ok((&mut self.ecx as *mut [u8], 0, 1)),
            TokenValue::EDX => return Ok((&mut self.edx as *mut [u8], 0, 4)),
            TokenValue::DX => return Ok((&mut self.edx as *mut [u8], 0, 2)),
            TokenValue::DH => return Ok((&mut self.edx as *mut [u8], 1, 1)),
            TokenValue::DL => return Ok((&mut self.edx as *mut [u8], 0, 1)),
            TokenValue::ESI => return Ok((&mut self.esi as *mut [u8], 0, 4)),
            TokenValue::SI => return Ok((&mut self.esi as *mut [u8], 0, 2)),
            TokenValue::EDI => return Ok((&mut self.edi as *mut [u8], 0, 4)),
            TokenValue::DI => return Ok((&mut self.edi as *mut [u8], 0, 2)),
            TokenValue::ESP => return Ok((&mut self.esp as *mut [u8], 0, 4)),
            TokenValue::SP => return Ok((&mut self.esp as *mut [u8], 0, 2)),
            TokenValue::EBP => return Ok((&mut self.ebp as *mut [u8], 0, 4)),
            TokenValue::BP => return Ok((&mut self.ebp as *mut [u8], 0, 2)),
            _ => return Err("Flag registers can not be used as source!".to_string()),
        }
    }

    fn get_value((pointer, start, size): (*mut [u8], usize, usize)) -> u32 {
        let mut value = [0; 4];

        unsafe {
            let (left, _right) = value.split_at_mut(size);
            left.copy_from_slice(&(*pointer)[start..start + size]);
        }

        u32::from_le_bytes(value)
    }

    fn set_value(&self, (pointer, start, size): (*mut [u8], usize, usize), value: u32) {
        unsafe {
            let (_left, right) = (*pointer).split_at_mut(start);
            let (left, _right) = right.split_at_mut(size);
            left.copy_from_slice(&value.to_le_bytes()[0..size]);
        }
    }

    fn parse_immediate_data(&mut self) -> (*mut [u8], usize, usize) {
        let sign = self.validate_token_value(TokenValue::MINUS, true);

        let mut value: i64 = self.text[self.get_eip()].get_int_value().try_into().unwrap();
        self.go_from_here(1);

        if sign {
            value = -value;
        }

        let size;

        if value >=0 {
            if value <= std::u8::MAX as i64 {
                size = 1;
            } else if value <= std::u16::MAX as i64 {
                size = 2;
            } else if value <= std::u32::MAX as i64 {
                size = 4;
            } else {
                panic!("Syntax Error: {} Integer literal: \"{}\" is too big!", self.text[self.get_eip() -
                        1].get_token_location().to_string(), self.text[self.get_eip() - 1].get_token_name());
            }
        } else {
            if value >= std::i8::MIN as i64 {
                size = 1;
            } else if value >= std::i16::MIN as i64 {
                size = 2;
            } else if value >= std::i32::MIN as i64 {
                size = 4;
            } else {
                panic!("Syntax Error: {} Integer literal: \"{}\" is too small!", self.text[self.get_eip() -
                        1].get_token_location().to_string(), self.text[self.get_eip() - 1].get_token_name());
            }
        }

        let pointer = Box::into_raw(Box::new((value as u32).to_le_bytes()));

        (pointer, 0, size)
    }

    fn parse_binary_operation(&mut self, lhs: u32, precedence: i32) -> u32 {
        let mut result = lhs;

        loop {
            let current_precedence = self.text[self.get_eip()].get_precedence();

            if current_precedence < precedence {
                return result;
            }

            let operation = self.text[self.get_eip()].get_token_value();
            self.go_from_here(1);

            let mut rhs = match self.text[self.get_eip()].get_token_type() {
                TokenType::REGISTER => {
                    VM::get_value(self.parse_register().unwrap())
                },
                TokenType::IMMEDIATE_DATA => {
                    self.go_from_here(1);
                    self.text[self.get_eip() - 1].get_int_value()
                },
                _ => {
                    self.error_report(&format!("Unexpected token: {}", self.text[self.get_eip()].get_token_name()));
                    std::u32::MAX
                },
            };

            let next_precedence = self.text[self.get_eip()].get_precedence();

            if current_precedence < next_precedence {
                rhs = self.parse_binary_operation(rhs, current_precedence + 1);
            }

            result = match operation {
                TokenValue::PLUS => lhs + rhs,
                TokenValue::MINUS => lhs - rhs,
                TokenValue::TIMES => lhs * rhs,
                _ => std::u32::MAX,
            };
        }
    }

    fn parse_address(&mut self) -> usize {
        let lhs = match self.text[self.get_eip()].get_token_type() {
            TokenType::REGISTER => {
                    VM::get_value(self.parse_register().unwrap())
            },
            TokenType::IMMEDIATE_DATA => {
                self.go_from_here(1);
                self.text[self.get_eip() - 1].get_int_value()
            },
            _ => {
                let value;
                if self.text[self.get_eip()].get_token_value() == TokenValue::MINUS {
                    self.go_from_here(2);
                    value = self.text[self.get_eip() - 1].get_int_value().overflowing_neg().0;
                } else {
                    self.error_report(&format!("Unexpected token: {}", self.text[self.get_eip()].get_token_name()));
                    value = std::u32::MAX;
                }

                value
            },
        };

        self.parse_binary_operation(lhs, 0) as usize
    }

    fn parse_memory(&mut self) -> Result<(*mut [u8], usize, usize), String> {
        let size = match self.text[self.get_eip()].get_token_value() {
            TokenValue::BYTE => 1,
            TokenValue::WORD => 2,
            TokenValue::DWORD => 4,
            _ => 0,
        };

        self.go_from_here(1);

        if !self.expect_token_value(TokenValue::PTR, "ptr".to_string(), true) {
            return Err("Missing \"PTR\" !".to_string());
        }

        if !self.expect_token_value(TokenValue::LBRACK, "[".to_string(), true) {
            return Err("Missing left brack '[' !".to_string());
        }

        let mem_add: usize = match self.parse_address().try_into() {
            Ok(mem_add) => mem_add,
            Err(err) => panic!("Invaild memory address: {}", err),
        };

        if !self.expect_token_value(TokenValue::RBRACK, "]".to_string(), true) {
            return Err("Missing right brack ']' !".to_string());
        }

        return Ok((&mut self.memory as *mut [u8], mem_add, size));
    }

    fn parse_source(&mut self) -> Result<(*mut [u8], usize, usize), String> {
        match self.text[self.get_eip()].get_token_value() {
            TokenValue::BYTE | TokenValue::WORD | TokenValue::DWORD => {
                return self.parse_memory();
            },
            _ => {},
        }

        if self.validate_token_type(TokenType::REGISTER, false) {
            return self.parse_register();
        } else if self.validate_token_type(TokenType::IMMEDIATE_DATA, false) ||
            self.validate_token_value(TokenValue::MINUS, false) {
            return Ok(self.parse_immediate_data());
        } else {
            self.error_report(&format!("Unexpected token: {}", self.text[self.get_eip()].get_token_name()));
            return Err(format!("{}: Unexpected token: {}", self.text[self.get_eip()].get_token_location().to_string(),
                        self.text[self.get_eip()].get_token_name()));
        }
    }

    fn parse_destination(&mut self) -> Result<(*mut [u8], usize, usize), String> {
        match self.text[self.get_eip()].get_token_value() {
            TokenValue::BYTE | TokenValue::WORD | TokenValue::DWORD => {
                return self.parse_memory();
            },
            _ => {},
        }

        if self.validate_token_type(TokenType::REGISTER, false) {
            return self.parse_register();
        } else {
            self.error_report(&format!("Unexpected token: {}", self.text[self.get_eip()].get_token_name()));
            return Err(format!("{}: Unexpected token: {}", self.text[self.get_eip()].get_token_location().to_string(),
                        self.text[self.get_eip()].get_token_name()));
        }
    }

    /// `mov` instruction
    ///
    /// mov &lt;reg&gt;, &lt;reg&gt;
    ///
    /// mov &lt;reg&gt;, &lt;mem&gt;
    ///
    /// mov &lt;mem&gt;, &lt;reg&gt;
    ///
    /// mov &lt;reg&gt;, &lt;const&gt;
    ///
    /// mov &lt;mem&gt;, &lt;const&gt;
    fn mov(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let value;
        if self.validate_token_type(TokenType::IMMEDIATE_DATA, false) || self.validate_token_value(TokenValue::MINUS,
                false) {
            let data = self.parse_immediate_data();

            if destination.2 < data.2 {
                panic!("Syntax Error: {} The destination is {} bytes, but source is {} bytes", self.text[self.get_eip() -
                        1].get_token_location().to_string(), destination.2, data.2);
            }

            let mut bytes = [0; 4];
            unsafe { bytes.copy_from_slice(&(*data.0)[0..4]); }
            value = u32::from_le_bytes(bytes);
        } else {
            let source = self.parse_source().unwrap();

            if destination.2 != source.2 {
                panic!("Syntax Error: {} The destination is {} bytes, but source is {} bytes", self.text[self.get_eip() -
                        1].get_token_location().to_string(), destination.2, source.2);
            }

            value = VM::get_value(source);
        }

        self.set_value(destination, value);
    }

    /// `movsx` instruction
    ///
    /// movsx &lt;reg16&gt;, &lt;reg8&gt;
    ///
    /// movsx &lt;reg16&gt;, &lt;mem8&gt;
    ///
    /// movsx &lt;reg32&gt;, &lt;reg8&gt;
    ///
    /// movsx &lt;reg32&gt;, &lt;mem8&gt;
    ///
    /// movsx &lt;reg32&gt;, &lt;reg16&gt;
    ///
    /// movsx &lt;reg32&gt;, &lt;mem16&gt;
    fn movsx(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::REGISTER, "register".to_string(), false) {
            return;
        }

        let destination = self.parse_register().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        if !self.validate_token_type(TokenType::REGISTER, false) && !self.validate_token_value(TokenValue::BYTE, false)
            && !self.validate_token_value(TokenValue::WORD, false) && !self.validate_token_value(TokenValue::DWORD,
                    false) {
            return;
        }

        let source = self.parse_source().unwrap();

        if destination.2 <= source.2 {
            panic!("Syntax Error: {} The destination is {} bytes, but source is {} bytes", self.text[self.get_eip() -
                    1].get_token_location().to_string(), destination.2, source.2);
        }

        let mut bytes;
        unsafe {
            if (*source.0)[source.1 + source.2 - 1] >= 128 {
                bytes = [0xff; 4];
            } else {
                bytes = [0x00; 4];
            }

            let (left, _right) = bytes.split_at_mut(source.2);
            left.copy_from_slice(&(*source.0)[source.1..source.1 + source.2]);
        }

        self.set_value(destination, u32::from_le_bytes(bytes));
    }

    /// `movzx` instruction
    ///
    /// movzx &lt;reg16&gt;, &lt;reg8&gt;
    ///
    /// movzx &lt;reg16&gt;, &lt;mem8&gt;
    ///
    /// movzx &lt;reg32&gt;, &lt;reg8&gt;
    ///
    /// movzx &lt;reg32&gt;, &lt;mem8&gt;
    ///
    /// movzx &lt;reg32&gt;, &lt;reg16&gt;
    ///
    /// movzx &lt;reg32&gt;, &lt;mem16&gt;
    fn movzx(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::REGISTER, "register".to_string(), false) {
            return;
        }

        let destination = self.parse_register().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        if !self.validate_token_type(TokenType::REGISTER, false) && !self.validate_token_value(TokenValue::BYTE, false)
            && !self.validate_token_value(TokenValue::WORD, false) && !self.validate_token_value(TokenValue::DWORD,
                    false) {
            return;
        }

        let source = self.parse_source().unwrap();

        if destination.2 <= source.2 {
            panic!("Syntax Error: {} The destination is {} bytes, but source is {} bytes", self.text[self.get_eip() -
                    1].get_token_location().to_string(), destination.2, source.2);
        }

        let mut bytes = [0; 4];
        unsafe {

            let (left, _right) = bytes.split_at_mut(source.2);
            left.copy_from_slice(&(*source.0)[source.1..source.1 + source.2]);
        }

        self.set_value(destination, u32::from_le_bytes(bytes));
    }

    fn set_cf_and_of(&mut self, result: u32, size: usize) {
        let tmp = result as i32;

        match size {
            1 => {
                if result < std::u8::MIN as u32 || result > std::u8::MAX as u32 {
                    self.cf = true;
                }

                if tmp < std::i8::MIN as i32 || tmp > std::i8::MAX as i32 {
                    self.of = true;
                }
            },
            2 => {
                if result < std::u16::MIN as u32 || result > std::u16::MAX as u32{
                    self.cf = true;
                }

                if tmp < std::i16::MIN as i32 || tmp > std::i16::MAX as i32 {
                    self.of = true;
                }
            },
            4 => {},
            _ => panic!("Invaild length: {}", size),
        }
    }

    fn set_sf_and_zf(&mut self, result: u32) {
        let tmp = result as i32;

        if tmp > 0 {
            self.sf = false;
            self.zf = false;
        } else if tmp == 0 {
            self.sf = false;
            self.zf = true;
        } else {
            self.sf = true;
            self.zf = false;
        }
    }

    /// binary operation, including `add`, `sub`, `and`, `or`, `xor`.
    ///
    /// bop &lt;reg&gt;, &lt;reg&gt;
    ///
    /// bop &lt;reg&gt;, &lt;mem&gt;
    ///
    /// bop &lt;mem&gt;, &lt;reg&gt;
    ///
    /// bop &lt;reg&gt;, &lt;con&gt;
    ///
    /// bop &lt;mem&gt;, &lt;con&gt;
    fn binary_operation(&mut self) {
        let instruction = self.text[self.get_eip()].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let source = self.parse_source().unwrap();

        if source.2 != 0 && destination.2 < source.2 {
            panic!("Syntax Error: {} The destination is {} bytes, but source is {} bytes", self.text[self.get_eip() -
                    1].get_token_location().to_string(), destination.2, source.2);
        }

        let first_operand = VM::get_value(destination);
        let second_operand = VM::get_value(source);
        let result;
        match instruction.get_token_value() {
            TokenValue::ADD => {
                let pair = first_operand.overflowing_add(second_operand);
                result = pair.0;
                self.cf = pair.1;
                self.of = (first_operand as i32).overflowing_add(second_operand as i32).1;
                self.set_cf_and_of(result, destination.2);
            },
            TokenValue::SUB => {
                let pair = first_operand.overflowing_sub(second_operand);
                result = pair.0;
                self.cf = pair.1;
                self.of = (first_operand as i32).overflowing_add(second_operand as i32).1;
                self.set_cf_and_of(result, destination.2);
            },
            TokenValue::AND => {
                result = first_operand & second_operand;
                self.cf = false;
                self.of = false;
            },
            TokenValue::OR => {
                result = first_operand | second_operand;
                self.cf = false;
                self.of = false;
            },
            TokenValue::XOR => {
                result = first_operand ^ second_operand;
                self.cf = false;
                self.of = false;
            },
            _ => {
                result = std::u32::MAX;
                self.error_report(&format!("Unexpected instruction: {}", instruction.get_token_name()));
            },
        };

        self.set_sf_and_zf(result);

        self.set_value(destination, result);
    }

    /// `mul` instruction
    ///
    /// mul &lt;reg8&gt;
    ///
    /// mul &lt;mem8&gt;
    ///
    /// mul &lt;reg16&gt;
    ///
    /// mul &lt;mem16&gt;
    ///
    /// mul &lt;reg32&gt;
    ///
    /// mul &lt;mem32&gt;
    fn mul(&mut self) {
        self.go_from_here(1);

        let multiplier = self.parse_destination().unwrap();

        match multiplier.2 {
            1 => {
                let multiplicand: u32 = self.eax[0].try_into().unwrap();
                let result = multiplicand.wrapping_mul(VM::get_value(multiplier));
                let old_eax = &mut self.eax as *mut [u8];
                self.set_value((old_eax, 0, 2), result);
                self.cf = result > 255;
                self.of = self.cf;
                self.set_sf_and_zf(result);
            },
            2 => {
                let mut bytes = [0; 2];
                &bytes.copy_from_slice(&self.eax[0..2]);
                let multiplicand: u32 = u16::from_le_bytes(bytes).try_into().unwrap();
                let result = multiplicand.wrapping_mul(VM::get_value(multiplier));
                let old_eax = &mut self.eax as *mut [u8];
                let old_edx = &mut self.edx as *mut [u8];
                self.set_value((old_eax, 0, 2), result);
                self.set_value((old_edx, 0, 2), result >> 16);
                self.cf = result >= (1u32 << 16);
                self.of = self.cf;
                self.set_sf_and_zf(result);
            },
            4 => {
                let multiplicand: u64 = u32::from_le_bytes(self.eax).try_into().unwrap();
                let result = multiplicand.wrapping_mul(VM::get_value(multiplier) as u64);
                let old_eax = &mut self.eax as *mut [u8];
                let old_edx = &mut self.edx as *mut [u8];
                self.set_value((old_eax, 0, 4), result as u32);
                self.set_value((old_edx, 0, 4), (result >> 32) as u32);
                self.cf = result >= (1u64 << 32);
                self.of = self.cf;

                let tmp = result as i64;

                if tmp > 0 {
                    self.sf = false;
                    self.zf = false;
                } else if tmp == 0 {
                    self.sf = false;
                    self.zf = true;
                } else {
                    self.sf = true;
                    self.zf = false;
                }
            },
            _ => {},
        }
    }

    /// `imul` instruction, only support for integer.
    ///
    /// imul &lt;reg32&gt;, &lt;reg32&gt;
    ///
    /// imul &lt;reg32&gt;, &lt;mem&gt;
    ///
    /// imul &lt;reg32&gt;, &lt;reg32&gt;, &lt;con&gt;
    ///
    /// imul &lt;reg32&gt;, &lt;mem&gt;, &lt;con&gt;
    fn imul(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::REGISTER, "register".to_string(), false) {
            return;
        }

        let destination = self.parse_register().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let first_operand = self.parse_destination().unwrap();
        let second_operand;
        let result;

        if self.validate_token_value(TokenValue::COMMA, true) {
            if !self.validate_token_type(TokenType::IMMEDIATE_DATA, false) {
                return;
            }

            second_operand = self.text[self.get_eip()].get_int_value();
            self.go_from_here(1);

            let pair = VM::get_value(first_operand).overflowing_mul(second_operand);
            result = pair.0;
            self.cf = pair.1;

            // self.set_flag(result, destination.2);

            self.set_value(destination, result);
        } else {
            let pair = VM::get_value(destination).overflowing_mul(VM::get_value(first_operand));
            result = pair.0;
            self.cf = pair.1;

            self.set_value(destination, result);
        }
    }

    /// `div` instruction
    ///
    /// div &lt;reg8&gt;
    ///
    /// div &lt;mem8&gt;
    ///
    /// div &lt;reg16&gt;
    ///
    /// div &lt;mem16&gt;
    ///
    /// div &lt;reg32&gt;
    ///
    /// div &lt;mem32&gt;
    fn div(&mut self) {
        let is_unsigned = self.validate_token_value(TokenValue::MUL, true);

        let divisor = self.parse_destination().unwrap();

        match divisor.2 {
            1 => {
                let mut bytes = [0; 2];
                &bytes.copy_from_slice(&self.eax[0..2]);
                let dividend = u16::from_le_bytes(bytes);
                let quotient;
                let remainder;

                if is_unsigned {
                    quotient = dividend.wrapping_div(VM::get_value(divisor) as u16);
                    remainder = dividend.wrapping_rem(VM::get_value(divisor) as u16);
                } else {
                    quotient = (dividend as i16).wrapping_div(VM::get_value(divisor) as i16) as u16;
                    remainder = (dividend as i16).wrapping_rem(VM::get_value(divisor) as i16) as u16;
                }

                let old_eax = &mut self.eax as *mut [u8];
                let old_edx = &mut self.edx as *mut [u8];
                self.set_value((old_eax, 0, 1), quotient as u32);
                self.set_value((old_edx, 1, 1), remainder as u32);
            },
            2 => {
                let mut bytes = [0; 4];
                {
                    let (left, right) = bytes.split_at_mut(2);
                    left.copy_from_slice(&self.eax[0..2]);
                    right.copy_from_slice(&self.edx[0..2]);
                }

                let dividend = u32::from_le_bytes(bytes);
                let quotient;
                let remainder;

                if is_unsigned {
                    quotient = dividend.wrapping_div(VM::get_value(divisor));
                    remainder = dividend.wrapping_rem(VM::get_value(divisor));
                } else {
                    quotient = (dividend as i32).wrapping_div(VM::get_value(divisor) as i32) as u32;
                    remainder = (dividend as i32).wrapping_rem(VM::get_value(divisor) as i32) as u32;
                }

                let old_eax = &mut self.eax as *mut [u8];
                let old_edx = &mut self.edx as *mut [u8];
                self.set_value((old_eax, 0, 2), quotient);
                self.set_value((old_edx, 0, 2), remainder);
            },
            4 => {
                let mut bytes = [0; 8];
                {
                    let (left, right) = bytes.split_at_mut(4);
                    left.copy_from_slice(&self.eax);
                    right.copy_from_slice(&self.edx);
                }

                let dividend = u64::from_le_bytes(bytes);
                let quotient;
                let remainder;

                if is_unsigned {
                    quotient = dividend.wrapping_div(VM::get_value(divisor) as u64);
                    remainder = dividend.wrapping_rem(VM::get_value(divisor) as u64);
                } else {
                    quotient = (dividend as i64).wrapping_div(VM::get_value(divisor) as i64) as u64;
                    remainder = (dividend as i64).wrapping_rem(VM::get_value(divisor) as i64) as u64;
                }

                let old_eax = &mut self.eax as *mut [u8];
                let old_edx = &mut self.edx as *mut [u8];
                self.set_value((old_eax, 0, 4), quotient as u32);
                self.set_value((old_edx, 0, 4), remainder as u32);
            },
            _ => {},
        }
    }

    /// unary operation, including `inc`, `dec`, `not`, `neg`.
    ///
    /// uop &lt;reg32&gt;
    ///
    /// uop &lt;mem&gt;
    fn unary_operation(&mut self) {
        let instruction = self.text[self.get_eip()].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        let operand = VM::get_value(destination);
        let result;
        match instruction.get_token_value() {
            TokenValue::INC => {
                result = operand.overflowing_add(1).0;
                self.of = (operand as i32).overflowing_add(1).1;
                self.set_cf_and_of(result, destination.2);
            },
            TokenValue::DEC => {
                result = operand.overflowing_sub(1).0;
                self.of = (operand as i32).overflowing_sub(1).1;
                self.set_cf_and_of(result, destination.2);
            },
            TokenValue::NOT => {
                result = !VM::get_value(destination);
            },
            TokenValue::NEG => {
                let pair = VM::get_value(destination).overflowing_neg();
                result = pair.0;
                self.cf = pair.1;
            },
            _ => {
                result = std::u32::MAX;
                self.error_report(&format!("Unexpected instruction: {}", instruction.get_token_name()));
            },
        };

        self.set_sf_and_zf(result);

        self.set_value(destination, result);
    }

    fn bitshift(&mut self) {
        let instruction = self.text[self.get_eip()].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immediate data".to_string(), false) {
            return;
        }

        let operand = VM::get_value(destination) as u64;
        let count = self.text[self.get_eip()].get_int_value();
        self.go_from_here(1);

        let result;
        match instruction.get_token_value() {
            TokenValue::SHL => {
                result = operand.wrapping_shl(count);
                self.cf = result & (1u64 << (8 * destination.2)) > 0;
                self.of = (result & (1u64 << (8 * destination.2 - 1)) > 0) ^ self.cf;
            },
            TokenValue::SHR => {
                result = operand.wrapping_shr(count);
                self.cf = (result & 1u64) > 0;
                self.of = operand >= (1u64 << (8 * destination.2 - 1));
            },
            TokenValue::SAR => {
                let tmp: i64 = (operand as i32).try_into().unwrap();
                result = tmp.wrapping_shr(count) as u64;
                self.cf = (result & 1u64) > 0;
                self.of = false;
            },
            _ => {
                result = std::u64::MAX;
                self.cf = false;
            },
        };

        self.set_sf_and_zf(result as u32);

        self.set_value(destination, result as u32);
    }

    /// `push` instruction
    ///
    /// push &lt;reg32&gt;
    ///
    /// push &lt;mem&gt;
    ///
    /// push &lt;con32&gt;
    fn push(&mut self) {
        self.go_from_here(1);

        let source = self.parse_source().unwrap();

        let old_esp = &mut self.esp as *mut [u8];
        let old_stack = &mut self.stack as *mut [u8];

        let new_esp = VM::get_value((old_esp, 0, 4)) - source.2 as u32;
        self.set_value((old_esp, 0, 4), new_esp);
        self.set_value((old_stack, new_esp as usize, source.2), VM::get_value(source));
    }

    /// `pop` instruction
    ///
    /// pop &lt;reg32&gt;
    ///
    /// pop &lt;mem&gt;
    fn pop(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        let old_esp = &mut self.esp as *mut [u8];

        let value = VM::get_value((&mut self.stack as *mut [u8], VM::get_value((old_esp, 0, 4)) as usize, destination.2));
        self.set_value(destination, value);
        let new_esp = VM::get_value((old_esp, 0, 4)) + destination.2 as u32;
        self.set_value((old_esp, 0, 4), new_esp);
    }

    /// `cmp` instruction
    /// cmp &lt;reg&gt;, &lt;reg&gt;
    ///
    /// cmp &lt;reg&gt;, &lt;mem&gt;
    ///
    /// cmp &lt;mem&gt;, &lt;reg&gt;
    ///
    /// cmp &lt;reg&gt;, &lt;con&gt;
    fn cmp(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();
        let first_operand = VM::get_value(destination);

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let source = self.parse_source().unwrap();
        let second_operand = VM::get_value(source);

        if first_operand > second_operand {
            self.cf = false;
            self.zf = false;
        } else if first_operand == second_operand {
            self.cf = false;
            self.zf = true;
        } else {
            self.cf = true;
            self.zf = false;
        }

        let mut bytes;
        unsafe {
            if (*destination.0)[destination.1 + destination.2 - 1] >= 128 {
                bytes = [0xff; 4];
            } else {
                bytes = [0x00; 4];
            }

            let (left, _right) = bytes.split_at_mut(destination.2);
            left.copy_from_slice(&(*destination.0)[destination.1..destination.1 + destination.2]);
        }
        let first_operand = i32::from_le_bytes(bytes);

        unsafe {
            if (*source.0)[source.1 + source.2 - 1] >= 128 {
                bytes = [0xff; 4];
            } else {
                bytes = [0x00; 4];
            }

            let (left, _right) = bytes.split_at_mut(source.2);
            left.copy_from_slice(&(*source.0)[source.1..source.1 + source.2]);
        }
        let second_operand = i32::from_le_bytes(bytes);
        self.sf = first_operand < second_operand;

        let tmp = first_operand - second_operand;
        self.of = (first_operand * second_operand <= 0) & (tmp * second_operand > 0);
    }

    fn jump(&mut self) {
        let instruction = self.text[self.get_eip()].to_owned();

        self.go_from_here(1);

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immediate data".to_string(), false) {
            return;
        }

        let displacement = self.text[self.get_eip()].get_int_value() as i32;
        self.go_from_here(1);

        match instruction.get_token_value() {
            TokenValue::JMP => {
                self.go_from_here(displacement);
            },
            TokenValue::JE => {
                if self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JNE => {
                if !self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JG => {
                if !self.zf && self.sf == self.of {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JGE => {
                if self.sf == self.of {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JL => {
                if self.sf != self.of {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JLE => {
                if self.zf || self.sf != self.of {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JA => {
                if !self.cf && !self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JAE => {
                if !self.cf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JB => {
                if self.cf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JBE => {
                if self.cf || self.zf {
                    self.go_from_here(displacement);
                }
            },
            _ => {},
        }
    }

    /// `call` instruction
    ///
    /// call &lt;label&gt;
    fn call(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immedate data".to_string(), false) {
            return;
        }

        let displacement = self.text[self.get_eip()].get_int_value() as i32;
        self.go_from_here(1);

        let old_esp = &mut self.esp as *mut [u8];
        let old_stack = &mut self.stack as *mut [u8];

        let new_esp = VM::get_value((old_esp, 0, 4)) - 4;
        self.set_value((old_esp, 0, 4), new_esp);
        self.set_value((old_stack, new_esp as usize, 4), self.get_eip() as u32);

        self.depth = self.depth + 1;

        self.go_from_here(displacement);
    }

    /// `ret` instruction
    fn ret(&mut self) {
        self.go_from_here(1);

        if self.depth > 1 {
            let old_esp = &mut self.esp as *mut [u8];
            let old_stack = &mut self.stack as *mut [u8];
            let old_eip = &mut self.eip as *mut [u8];

            let value = VM::get_value((old_stack, VM::get_value((old_esp, 0, 4)) as usize, 4));
            self.set_value((old_eip, 0, 4), value);
            let new_esp = VM::get_value((old_esp, 0, 4)) + 4;
            self.set_value((old_esp, 0, 4), new_esp);
        }

        self.depth = self.depth - 1;
    }

    /// `enter` instruction
    fn enter(&mut self) {
        self.go_from_here(1);

        let old_esp = &mut self.esp as *mut [u8];
        let old_stack = &mut self.stack as *mut [u8];
        let old_ebp = &mut self.ebp as *mut [u8];

        let new_esp = VM::get_value((old_esp, 0, 4)) - 4;
        self.set_value((old_esp, 0, 4), new_esp);
        self.set_value((old_stack, new_esp as usize, 4), VM::get_value((old_ebp, 0, 4)));

        self.ebp = self.esp;
    }

    /// `leave` instruction
    fn leave(&mut self) {
        self.go_from_here(1);

        self.esp = self.ebp;

        let old_esp = &mut self.esp as *mut [u8];
        let old_stack = &mut self.stack as *mut [u8];
        let old_ebp = &mut self.ebp as *mut [u8];

        let value = VM::get_value((old_stack, VM::get_value((old_esp, 0, 4)) as usize, 4));
        self.set_value((old_ebp, 0, 4), value);
        let new_esp = VM::get_value((old_esp, 0, 4)) + 4;
        self.set_value((old_esp, 0, 4), new_esp);
    }

    pub fn get_eax(&self) -> u32 {
        u32::from_le_bytes(self.eax)
    }

    pub fn get_ebx(&self) -> u32 {
        u32::from_le_bytes(self.ebx)
    }

    pub fn get_ecx(&self) -> u32 {
        u32::from_le_bytes(self.ecx)
    }

    pub fn get_edx(&self) -> u32 {
        u32::from_le_bytes(self.edx)
    }

    pub fn get_text(&self) -> Vec<Token> {
        self.text.to_owned()
    }

    /// Run vm.
    ///
    /// # Examples
    ///
    /// ```
    /// let vm = VM::new("./test.asm".to_string());
    /// vm.run();
    /// ```
    pub fn run(&mut self) {
        self.preprocess();

        loop {
            match self.text[self.get_eip()].get_token_type() {
                TokenType::INSTRUCTION => {
                    match self.text[self.get_eip()].get_token_value() {
                        TokenValue::MOV => self.mov(),
                        TokenValue::MOVSX => self.movsx(),
                        TokenValue::MOVZX => self.movzx(),
                        TokenValue::ADD | TokenValue::SUB | TokenValue::AND |
                            TokenValue::OR | TokenValue::XOR => self.binary_operation(),
                        TokenValue::MUL => self.mul(),
                        TokenValue::IMUL => self.imul(),
                        TokenValue::DIV | TokenValue::IDIV => self.div(),
                        TokenValue::INC | TokenValue::DEC | TokenValue::NOT | TokenValue::NEG => self.unary_operation(),
                        TokenValue::SHL | TokenValue::SHR | TokenValue::SAR => self.bitshift(),
                        TokenValue::PUSH => self.push(),
                        TokenValue::POP => self.pop(),
                        TokenValue::CMP => self.cmp(),
                        TokenValue::JMP | TokenValue::JE | TokenValue::JNE | TokenValue::JG | TokenValue::JGE | TokenValue::JL |
                            TokenValue::JLE | TokenValue::JA | TokenValue::JAE | TokenValue::JB | TokenValue::JBE => self.jump(),
                        TokenValue::CALL => self.call(),
                        TokenValue::RET => self.ret(),
                        TokenValue::ENTER => self.enter(),
                        TokenValue::LEAVE => self.leave(),
                        TokenValue::INT => break,
                        _ => self.error_report(&format!("Unexpected instruction: {}",
                                    self.text[self.get_eip()].get_token_name())),
                    }
                },
                TokenType::LABEL => {
                    self.go_from_here(2);
                },
                _ => self.error_report(&format!("Unexpected token: {}", self.text[self.get_eip()].get_token_name())),
            }

            if self.depth == 0 {
                break;
            }
        }
    }
}

