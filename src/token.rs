#![allow(dead_code)]

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
/// Type of token
pub enum TokenType {
    /// instruction, such as `mov`
    INSTRUCTION,
    /// register, such as `eax`
    REGISTER,
    /// delimiter, such as `,`
    DELIMITER,
    /// immediate date, such as `123
    IMMEDIATE_DATA,
    /// label, such as `main`
    LABEL,
    /// eof
    END_OF_FILE,
}

#[derive(Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
/// Value of token
pub enum TokenValue {
    /// instruction
    /// `mov`
    MOV,
    /// `add`
    ADD,
    /// `sub`
    SUB,
    /// `mul`
    MUL,
    /// `div`
    DIV,
    /// `and`
    AND,
    /// `or`
    OR,
    /// `xor`
    XOR,
    /// `not`
    NOT,
    /// `neg`
    NEG,
    /// `shl`
    SHL,
    /// `shr`
    SHR,
    /// `push`
    PUSH,
    /// `pop`
    POP,
    /// `cmp`
    CMP,
    /// `jmp`
    JMP,
    /// `je`
    JE,
    /// `jne`
    JNE,
    /// `jg`
    JG,
    /// `jge`
    JGE,
    /// `jl`
    JL,
    /// `jle`
    JLE,
    /// `call`
    CALL,
    /// `ret`
    RET,
    /// `int`
    INT,

    /// register
    /// `eax`
    EAX,
    /// `ebx`
    EBX,
    /// `ecx`
    ECX,
    /// `edx`
    EDX,
    /// `esi`
    ESI,
    /// `edi`
    EDI,
    /// `esp`
    ESP,
    /// `ebp`
    EBP,
    /// `eip`
    EIP,
    /// `zf`
    ZF,
    /// `sf`
    SF,

    /// demiliter
    /// `;`
    SEMICOLON,
    /// `,`
    COMMA,
    /// `[`
    LBRACK,
    /// `]'
    RBRACK,
    /// `:`
    COLON,

    /// immediate data
    INTEGER_LITERAL,
    /// label
    LABEL,

    /// eof
    END_OF_FILE,

    /// unknown token
    UNKNOWN,
}

impl TokenType {
    fn to_string(&self) -> String {
        let buffer = match self {
            TokenType::INSTRUCTION => "instruction",
            TokenType::REGISTER => "register",
            TokenType::DELIMITER => "delimiter",
            TokenType::IMMEDIATE_DATA => "immediate data",
            TokenType::LABEL => "label",
            TokenType::END_OF_FILE => "eof",
        };

        buffer.to_string()
    }
}

#[derive(Default)]
#[derive(Clone)]
/// Location of token
pub struct TokenLocation {
    source_file_name_: String,
    line_: i32,
    column_: i32
}

impl TokenLocation {
    pub fn new(souce_file_name: String, line: i32, column: i32) -> Self {
        TokenLocation {
            source_file_name_: souce_file_name,
            line_: line,
            column_: column,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}:", self.source_file_name_, self.line_, self.column_)
    }
}

#[derive(Clone)]
/// Lexical token
pub struct Token {
    type_: TokenType,
    value_: TokenValue,
    location_: TokenLocation,
    name_: String,
    int_value_: i32,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            type_: TokenType::INSTRUCTION,
            value_: TokenValue::MOV,
            location_: Default::default(),
            name_: "mov".to_string(),
            int_value_: 0,
        }
    }
}

impl Token {
    pub fn new_token(token_type: TokenType, token_value: TokenValue, loc: TokenLocation, name: String) -> Self {
        Token {
            type_: token_type,
            value_: token_value,
            location_: loc,
            name_: name,
            ..Default::default()
        }
    }

    pub fn new_int_token(loc: TokenLocation, name: String, int_value: i32) -> Self {
        Token {
            type_: TokenType::IMMEDIATE_DATA,
            value_: TokenValue::INTEGER_LITERAL,
            location_: loc,
            name_: name,
            int_value_: int_value,
            ..Default::default()
        }
    }

    pub fn get_token_location(&self) -> TokenLocation {
        self.location_.to_owned()
    }

    pub fn get_token_type(&self) -> TokenType {
        self.type_
    }

    pub fn get_token_value(&self) -> TokenValue {
        self.value_
    }

    pub fn get_token_name(&self) -> String {
       self.name_.to_owned()
    }

    pub fn get_int_value(&self) -> i32 {
        self.int_value_
    }

    pub fn set_token_type(&mut self, token_type: TokenType) {
        self.type_ = token_type;
    }

    /*
    pub fn get_token_value(&mut self, token_value: TokenValue) {
        self.value_ = token_value;
    }
    */

    pub fn set_int_value(&mut self, int_value: i32) {
        self.int_value_ = int_value;
    }

    pub fn to_string(&self) -> String {
        format!("{} Token Type: {}, Token Value: {}", self.location_.to_string(),self.type_.to_string(),
                self.name_)
    }

}
