use std::str::FromStr;
use crate::core::lexer;
use crate::core::lexer::TrainCar;
use crate::core::lexer::Tokenize;
use std::fmt::Display;
use crate::util::strcmp;

/// Transforms a VHDL integer `s` into a real unsigned number to be used in rust code.
/// 
/// Assumes the integer is valid under the following production rule:
/// - integer ::= digit { \[ underline ] digit }
fn interpret_integer(s: &str) -> usize {
    let mut chars = s.chars();
    let mut number = String::from(chars.next().expect("must have a lead-off digit"));
    while let Some(c) = chars.next() {
        if c != char_set::UNDERLINE {
            number.push(c);
        }
    }
    number.parse::<usize>().expect("integer can only contain 0..=9 or underline '_'")
}

#[derive(Debug, Clone, PartialOrd, Ord)]
pub enum Identifier {
    Basic(String),
    Extended(String),
}

use std::hash::Hasher;
use std::hash::Hash;
// @TODO test
impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Basic(id) => { id.to_lowercase().hash(state) },
            Self::Extended(id) => { id.hash(state) }
        }
    }
}

impl std::cmp::Eq for Identifier {}

impl Identifier {
    pub fn new() -> Self {
        Self::Basic(String::new())
    }

    // Returns the reference to the inner `String` struct.
    fn as_str(&self) -> &str {
        match self {
            Self::Basic(id) => id.as_ref(),
            Self::Extended(id) => id.as_ref(),
        }
    }

    /// Checks if `self` is an extended identifier or not.
    fn is_extended(&self) -> bool {
        match self {
            Self::Extended(_) => true,
            Self::Basic(_) => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Basic(id) => id.len(),
            Self::Extended(id) => id.len() + 2 + (id.chars().filter(|c| c == &'\\' ).count())
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum IdentifierError {
    Empty,
    InvalidFirstChar(char),
    CharsAfterDelimiter(String),
}

impl std::error::Error for IdentifierError {}

impl std::fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty identifier"),
            Self::CharsAfterDelimiter(s) => write!(f, "characters \'{}\' found following closing extended backslash, ", s),
            Self::InvalidFirstChar(c) => write!(f, "first character must be letter but found \'{}\'", c),
        }
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = TrainCar::new(s.chars());
        match chars.consume() {
            // check what type of identifier it is
            Some(c) => Ok(
                match c {
                    '\\' => {
                        let result = Self::Extended(VHDLToken::consume_literal(&mut chars, &char_set::BACKSLASH).unwrap());
                        // gather remaining characters
                        let mut rem = String::new();
                        while let Some(c) = chars.consume() {
                            rem.push(c);
                        }
                        match rem.is_empty() {
                            true => result,
                            false => return Err(Self::Err::CharsAfterDelimiter(rem)),
                        }
                    }
                    _ => {
                        // verify the first character was a letter
                        match char_set::is_letter(&c) { 
                            true => Self::Basic(VHDLToken::consume_value_pattern(&mut chars, Some(c), char_set::is_letter_or_digit).unwrap()),
                            false => return Err(Self::Err::InvalidFirstChar(c)),
                        } 
                    }
                }),
            None => Err(Self::Err::Empty)
        }
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // instantly not equal if not they are not of same type
        if self.is_extended() != other.is_extended() { return false };
        // compare with case sensitivity
        if self.is_extended() == true {
            self.as_str() == other.as_str()
        // compare without case sensitivity
        } else {
            strcmp::cmp_ignore_case(self.as_str(), other.as_str())
        }
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Extended(id) => write!(f, "\\{}\\", id.replace('\\', r#"\\"#)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Comment {
    Single(String),
    Delimited(String),
}

impl Comment {
    fn as_str(&self) -> &str {
        match self {
            Self::Single(note) => note.as_ref(),
            Self::Delimited(note) => note.as_ref(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Character(String);

impl Character {
    fn new(c: char) -> Self {
        Self(String::from(c))
    }

    fn as_str(&self) -> &str {
        &self.0.as_ref()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BitStrLiteral(String);

impl BitStrLiteral {
    /// Returns the reference to the inner `String` struct.
    fn as_str(&self) -> &str {
        &self.0.as_ref()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AbstLiteral {
    Decimal(String),
    Based(String),
}

impl AbstLiteral {
    fn as_str(&self) -> &str {
        match self {
            Self::Decimal(val) => val.as_ref(),
            Self::Based(val) => val.as_ref(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Abs,            // VHDL-1987 LRM - current 
    Access,         // VHDL-1987 LRM - current
    After,          // VHDL-1987 LRM - current
    Alias,          // VHDL-1987 LRM - current
    All,            // VHDL-1987 LRM - current
    And,            // VHDL-1987 LRM - current
    Architecture,   // VHDL-1987 LRM - current
    Array,          // VHDL-1987 LRM - current
    Assert,         // VHDL-1987 LRM - current
    Assume,
    // AssumeGuarantee "assume_guarantee" is omitted from VHDL-2019 LRM
    Attribute,      // VHDL-1987 LRM - current
    Begin,          // VHDL-1987 LRM - current
    Block,          // VHDL-1987 LRM - current
    Body,           // VHDL-1987 LRM - current
    Buffer,         // VHDL-1987 LRM - current
    Bus,            // VHDL-1987 LRM - current
    Case,           // VHDL-1987 LRM - current
    Component,      // VHDL-1987 LRM - current
    Configuration,  // VHDL-1987 LRM - current
    Constant,       // VHDL-1987 LRM - current
    Context,
    Cover,
    Default,
    Disconnect,     // VHDL-1987 LRM - current
    Downto,         // VHDL-1987 LRM - current
    Else,           // VHDL-1987 LRM - current
    Elsif,          // VHDL-1987 LRM - current
    End,            // VHDL-1987 LRM - current
    Entity,         // VHDL-1987 LRM - current
    Exit,           // VHDL-1987 LRM - current
    Fairness,
    File,           // VHDL-1987 LRM - current
    For,            // VHDL-1987 LRM - current
    Force,
    Function,       // VHDL-1987 LRM - current
    Generate,       // VHDL-1987 LRM - current
    Generic,        // VHDL-1987 LRM - current
    Group, 
    Guarded,        // VHDL-1987 LRM - current
    If,             // VHDL-1987 LRM - current
    Impure, 
    In,             // VHDL-1987 LRM - current
    Inertial, 
    Inout,          // VHDL-1987 LRM - current
    Is,             // VHDL-1987 LRM - current
    Label,          // VHDL-1987 LRM - current
    Library,        // VHDL-1987 LRM - current
    Linkage,        // VHDL-1987 LRM - current
    Literal,  
    Loop,           // VHDL-1987 LRM - current
    Map,            // VHDL-1987 LRM - current
    Mod,            // VHDL-1987 LRM - current
    Nand,           // VHDL-1987 LRM - current
    New,            // VHDL-1987 LRM - current
    Next,           // VHDL-1987 LRM - current
    Nor,            // VHDL-1987 LRM - current
    Not,            // VHDL-1987 LRM - current
    Null,           // VHDL-1987 LRM - current
    Of,             // VHDL-1987 LRM - current
    On,             // VHDL-1987 LRM - current
    Open,           // VHDL-1987 LRM - current
    Or,             // VHDL-1987 LRM - current
    Others,         // VHDL-1987 LRM - current
    Out,            // VHDL-1987 LRM - current
    Package,        // VHDL-1987 LRM - current
    Parameter, 
    Port,           // VHDL-1987 LRM - current
    Postponed, 
    Private,
    Procedure,      // VHDL-1987 LRM - current
    Process,        // VHDL-1987 LRM - current
    Property, 
    Protected, 
    Pure,
    Range,          // VHDL-1987 LRM - current
    Record,         // VHDL-1987 LRM - current
    Register,       // VHDL-1987 LRM - current
    Reject,
    Release,
    Rem,            // VHDL-1987 LRM - current
    Report,         // VHDL-1987 LRM - current
    Restrict, 
    // RestrictGuarantee "restrict_guarantee" is omitted from VHDL-2019 LRM
    Return,         // VHDL-1987 LRM - current
    Rol, 
    Ror,
    Select,         // VHDL-1987 LRM - current
    Sequence, 
    Severity,       // VHDL-1987 LRM - current
    Signal,         // VHDL-1987 LRM - current
    Shared, 
    Sla,
    Sll,
    Sra,
    Srl, 
    Strong, 
    Subtype,        // VHDL-1987 LRM - current
    Then,           // VHDL-1987 LRM - current
    To,             // VHDL-1987 LRM - current
    Transport,      // VHDL-1987 LRM - current
    Type,           // VHDL-1987 LRM - current
    Unaffected, 
    Units,          // VHDL-1987 LRM - current
    Until,          // VHDL-1987 LRM - current
    Use,            // VHDL-1987 LRM - current
    Variable,       // VHDL-1987 LRM - current
    View,
    Vmode, 
    Vpkg,
    Vprop, 
    Vunit,
    Wait,           // VHDL-1987 LRM - current
    When,           // VHDL-1987 LRM - current
    While,          // VHDL-1987 LRM - current
    With,           // VHDL-1987 LRM - current
    Xnor, 
    Xor,            // VHDL-1987 LRM - current
}

impl Keyword {
    /// Attempts to match the given string of characters `s` to a VHDL keyword.
    /// 
    /// Compares `s` against keywords using ascii lowercase comparison.
    fn match_keyword(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_ref() {
            "abs"           => Self::Abs, 
            "access"        => Self::Access, 
            "after"         => Self::After, 
            "alias"         => Self::Alias, 
            "all"           => Self::All, 
            "and"           => Self::And, 
            "architecture"  => Self::Architecture, 
            "array"         => Self::Array, 
            "assert"        => Self::Assert, 
            "assume"        => Self::Assume, 
            "attribute"     => Self::Attribute, 
            "begin"         => Self::Begin, 
            "block"         => Self::Block, 
            "body"          => Self::Body, 
            "buffer"        => Self::Buffer, 
            "bus"           => Self::Bus, 
            "case"          => Self::Case, 
            "component"     => Self::Component, 
            "configuration" => Self::Configuration, 
            "constant"      => Self::Constant, 
            "context"       => Self::Context, 
            "cover"         => Self::Cover, 
            "default"       => Self::Default, 
            "disconnect"    => Self::Disconnect, 
            "downto"        => Self::Downto, 
            "else"          => Self::Else, 
            "elsif"         => Self::Elsif, 
            "end"           => Self::End, 
            "entity"        => Self::Entity, 
            "exit"          => Self::Exit, 
            "fairness"      => Self::Fairness, 
            "file"          => Self::File, 
            "for"           => Self::For, 
            "force"         => Self::Force, 
            "function"      => Self::Function, 
            "generate"      => Self::Generate, 
            "generic"       => Self::Generic, 
            "group"         => Self::Group, 
            "guarded"       => Self::Guarded, 
            "if"            => Self::If, 
            "impure"        => Self::Impure, 
            "in"            => Self::In, 
            "inertial"      => Self::Inertial, 
            "inout"         => Self::Inout, 
            "is"            => Self::Is, 
            "label"         => Self::Label, 
            "library"       => Self::Library, 
            "linkage"       => Self::Linkage, 
            "literal"       => Self::Literal, 
            "loop"          => Self::Loop, 
            "map"           => Self::Map, 
            "mod"           => Self::Mod, 
            "nand"          => Self::Nand, 
            "new"           => Self::New, 
            "next"          => Self::Next, 
            "nor"           => Self::Nor, 
            "not"           => Self::Not, 
            "null"          => Self::Null, 
            "of"            => Self::Of, 
            "on"            => Self::On, 
            "open"          => Self::Open, 
            "or"            => Self::Or, 
            "others"        => Self::Others, 
            "out"           => Self::Out, 
            "package"       => Self::Package, 
            "parameter"     => Self::Parameter, 
            "port"          => Self::Port, 
            "postponed"     => Self::Postponed, 
            "private"       => Self::Private, 
            "procedure"     => Self::Procedure, 
            "process"       => Self::Process, 
            "property"      => Self::Property, 
            "protected"     => Self::Protected, 
            "pure"          => Self::Pure, 
            "range"         => Self::Range, 
            "record"        => Self::Record, 
            "register"      => Self::Register, 
            "reject"        => Self::Reject, 
            "release"       => Self::Release, 
            "rem"           => Self::Rem, 
            "report"        => Self::Report, 
            "restrict"      => Self::Restrict, 
            "return"        => Self::Return, 
            "rol"           => Self::Rol, 
            "ror"           => Self::Ror, 
            "select"        => Self::Select, 
            "sequence"      => Self::Sequence, 
            "severity"      => Self::Severity, 
            "signal"        => Self::Signal, 
            "shared"        => Self::Shared, 
            "sla"           => Self::Sla, 
            "sll"           => Self::Sll, 
            "sra"           => Self::Sra, 
            "srl"           => Self::Srl, 
            "strong"        => Self::Strong, 
            "subtype"       => Self::Subtype, 
            "then"          => Self::Then, 
            "to"            => Self::To, 
            "transport"     => Self::Transport, 
            "type"          => Self::Type, 
            "unaffected"    => Self::Unaffected, 
            "units"         => Self::Units, 
            "until"         => Self::Until, 
            "use"           => Self::Use, 
            "variable"      => Self::Variable, 
            "view"          => Self::View, 
            "vmode"         => Self::Vmode, 
            "vpkg"          => Self::Vpkg, 
            "vprop"         => Self::Vprop, 
            "vunit"         => Self::Vunit, 
            "wait"          => Self::Wait, 
            "when"          => Self::When, 
            "while"         => Self::While, 
            "with"          => Self::With, 
            "xnor"          => Self::Xnor, 
            "xor"           => Self::Xor, 
            _               => return None
        })
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Abs           => "abs",
            Self::Access        => "access",
            Self::After         => "after",
            Self::Alias         => "alias",
            Self::All           => "all",
            Self::And           => "and", 
            Self::Architecture  => "architecture",
            Self::Array         => "array",
            Self::Assert        => "assert",
            Self::Assume        => "assume",
            Self::Attribute     => "attribute",
            Self::Begin         => "begin",
            Self::Block         => "block",
            Self::Body          => "body",
            Self::Buffer        => "buffer",
            Self::Bus           => "bus",
            Self::Case          => "case", 
            Self::Component     => "component",
            Self::Configuration => "configuration",
            Self::Constant      => "constant", 
            Self::Context       => "context",
            Self::Cover         => "cover",
            Self::Default       => "default",
            Self::Disconnect    => "disconnect", 
            Self::Downto        => "downto",
            Self::Else          => "else", 
            Self::Elsif         => "elsif",
            Self::End           => "end",
            Self::Entity        => "entity", 
            Self::Exit          => "exit",
            Self::Fairness      => "fairness",
            Self::File          => "file",
            Self::For           => "for", 
            Self::Force         => "force",
            Self::Function      => "function",
            Self::Generate      => "generate", 
            Self::Generic       => "generic", 
            Self::Group         => "group", 
            Self::Guarded       => "guarded",
            Self::If            => "if",
            Self::Impure        => "impure", 
            Self::In            => "in",     
            Self::Inertial      => "inertial", 
            Self::Inout         => "inout", 
            Self::Is            => "is",
            Self::Label         => "label", 
            Self::Library       => "library", 
            Self::Linkage       => "linkage", 
            Self::Literal       => "literal", 
            Self::Loop          => "loop",
            Self::Map           => "map",
            Self::Mod           => "mod",
            Self::Nand          => "nand",
            Self::New           => "new", 
            Self::Next          => "next", 
            Self::Nor           => "nor", 
            Self::Not           => "not", 
            Self::Null          => "null",
            Self::Of            => "of",
            Self::On            => "on",
            Self::Open          => "open",
            Self::Or            => "or", 
            Self::Others        => "others", 
            Self::Out           => "out",
            Self::Package       => "package", 
            Self::Parameter     => "parameter", 
            Self::Port          => "port", 
            Self::Postponed     => "postponed", 
            Self::Private       => "private",
            Self::Procedure     => "procedure", 
            Self::Process       => "process", 
            Self::Property      => "property", 
            Self::Protected     => "protected", 
            Self::Pure          => "pure",
            Self::Range         => "range",
            Self::Record        => "record",    
            Self::Register      => "register",
            Self::Reject        => "reject",
            Self::Release       => "release",
            Self::Rem           => "rem",
            Self::Report        => "report",
            Self::Restrict      => "restrict", 
            Self::Return        => "return",
            Self::Rol           => "rol", 
            Self::Ror           => "ror",
            Self::Select        => "select", 
            Self::Sequence      => "sequence", 
            Self::Severity      => "severity",
            Self::Signal        => "signal", 
            Self::Shared        => "shared", 
            Self::Sla           => "sla",
            Self::Sll           => "sll",
            Self::Sra           => "sra",
            Self::Srl           => "srl", 
            Self::Strong        => "strong", 
            Self::Subtype       => "subtype",
            Self::Then          => "then",
            Self::To            => "to", 
            Self::Transport     => "transport", 
            Self::Type          => "type",
            Self::Unaffected    => "unaffected", 
            Self::Units         => "units",
            Self::Until         => "until",
            Self::Use           => "use",
            Self::Variable      => "variable", 
            Self::View          => "view",
            Self::Vmode         => "vmode", 
            Self::Vpkg          => "vpkg",
            Self::Vprop         => "vprop", 
            Self::Vunit         => "vunit",
            Self::Wait          => "wait", 
            Self::When          => "when", 
            Self::While         => "while", 
            Self::With          => "with",
            Self::Xnor          => "xnor", 
            Self::Xor           => "xor",
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Delimiter {
    Ampersand,      // &
    SingleQuote,    // '
    ParenL,         // (
    ParenR,         // )
    Star,           // *
    Plus,           // +
    Comma,          // ,
    Dash,           // -
    Dot,            // .
    FwdSlash,       // /
    Colon,          // :
    Terminator,     // ;
    Lt,             // <
    Eq,             // =
    Gt,             // >
    BackTick,       // `
    Pipe,           // | or ! VHDL-1993 LRM p180
    BrackL,         // [
    BrackR,         // ]
    Question,       // ?
    AtSymbol,       // @
    Arrow,          // =>
    DoubleStar,     // **
    VarAssign,      // :=
    Inequality,     // /=
    GTE,            // >=
    SigAssign,      // <=
    Box,            // <>
    SigAssoc,       // <=>
    CondConv,       // ??
    MatchEQ,        // ?=
    MatchNE,        // ?/=
    MatchLT,        // ?<
    MatchLTE,       // ?<=
    MatchGT,        // ?>
    MatchGTE,       // ?>=
    DoubleLT,       // <<
    DoubleGT,       // >>
}

impl Delimiter {
    /// Attempts to match the given string of characters `s` to a VHDL delimiter.
    fn transform(s: &str) -> Option<Self> {
        Some(match s {
            "&"     => Self::Ampersand,    
            "'"     => Self::SingleQuote,  
            "("     => Self::ParenL,       
            ")"     => Self::ParenR,       
            "*"     => Self::Star,         
            "+"     => Self::Plus,         
            ","     => Self::Comma,        
            "-"     => Self::Dash,         
            "."     => Self::Dot,          
            "/"     => Self::FwdSlash,     
            ":"     => Self::Colon,        
            ";"     => Self::Terminator,   
            "<"     => Self::Lt,           
            "="     => Self::Eq,           
            ">"     => Self::Gt,           
            "`"     => Self::BackTick,     
      "!" | "|"     => Self::Pipe,         
            "["     => Self::BrackL,       
            "]"     => Self::BrackR,       
            "?"     => Self::Question,     
            "@"     => Self::AtSymbol,     
            "=>"    => Self::Arrow,          
            "**"    => Self::DoubleStar,     
            ":="    => Self::VarAssign,      
            "/="    => Self::Inequality,     
            ">="    => Self::GTE,            
            "<="    => Self::SigAssign,      
            "<>"    => Self::Box,            
            "<=>"   => Self::SigAssoc,       
            "??"    => Self::CondConv,       
            "?="    => Self::MatchEQ,        
            "?/="   => Self::MatchNE,      
            "?<"    => Self::MatchLT,        
            "?<="   => Self::MatchLTE,       
            "?>"    => Self::MatchGT,        
            "?>="   => Self::MatchGTE,       
            "<<"    => Self::DoubleLT,       
            ">>"    => Self::DoubleGT,       
            _       => return None,
        })
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Ampersand     => "&",
            Self::SingleQuote   => "'",
            Self::ParenL        => "(",
            Self::ParenR        => ")",
            Self::Star          => "*",
            Self::Plus          => "+",
            Self::Comma         => ",",
            Self::Dash          => "-",
            Self::Dot           => ".",
            Self::FwdSlash      => "/",
            Self::Colon         => ":",
            Self::Terminator    => ";",
            Self::Lt            => "<",
            Self::Eq            => "=",
            Self::Gt            => ">",
            Self::BackTick      => "`",
            Self::Pipe          => "|",
            Self::BrackL        => "[",
            Self::BrackR        => "]",
            Self::Question      => "?",
            Self::AtSymbol      => "@",
            Self::Arrow         => "=>",
            Self::DoubleStar    => "**",
            Self::VarAssign     => ":=",
            Self::Inequality    => "/=",
            Self::GTE           => ">=",
            Self::SigAssign     => "<=",
            Self::Box           => "<>",
            Self::SigAssoc      => "<=>",
            Self::CondConv      => "??",
            Self::MatchEQ       => "?=",
            Self::MatchNE       => "?/=",
            Self::MatchLT       => "?<",
            Self::MatchLTE      => "?<=",
            Self::MatchGT       => "?>",
            Self::MatchGTE      => "?>=",
            Self::DoubleLT      => "<<",
            Self::DoubleGT      => ">>",
        }
    }
}

impl Display for Delimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VHDLToken {
    Comment(Comment),               // (String) 
    Identifier(Identifier),         // (String) ...can be general or extended (case-sensitive) identifier
    AbstLiteral(AbstLiteral),       // (String)
    CharLiteral(Character),         // (String)
    StrLiteral(String),             // (String)
    BitStrLiteral(BitStrLiteral),   // (String)
    Keyword(Keyword),
    Delimiter(Delimiter),
    EOF,
}

impl Display for VHDLToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Comment(note) => note.as_str(),
            Self::Identifier(id) => id.as_str(),
            Self::AbstLiteral(a) => a.as_str(),
            Self::CharLiteral(c) => c.as_str(),
            Self::StrLiteral(s) => s.as_str(),
            Self::BitStrLiteral(b) => b.as_str(),
            Self::Keyword(kw) => kw.as_str(),
            Self::Delimiter(d) => d.as_str(),
            Self::EOF => "EOF",
        })
    }
}

impl VHDLToken {
    /// Takes the identifier from the token.
    pub fn take_identifier(self) -> Option<Identifier> {
        match self {
            Self::Identifier(i) => Some(i),
            _ => None,
        }
    }

    /// Takes the keyword from the token.
    pub fn take_keyword(self) -> Option<Keyword> {
        match self {
            Self::Keyword(kw) => Some(kw),
            _ => None,
        }
    }

    /// Casts into a keyword.
    pub fn as_keyword(&self) -> Option<&Keyword> {
        match self {
            Self::Keyword(kw) => Some(kw),
            _ => None,
        }
    }

    /// Checks if the current token type `self` is a delimiter.
    fn is_delimiter(&self) -> bool {
        match self {
            Self::Delimiter(_) => true,
            _ => false,
        }
    }
    
    /// Attempts to match a string `s` to a valid delimiter.
    fn match_delimiter(s: &str) -> Result<Self, VHDLTokenError> {
        match Delimiter::transform(s) {
            Some(d) => Ok(VHDLToken::Delimiter(d)),
            None => Err(VHDLTokenError::Invalid(s.to_string()))
        }
    }
    
    /// Captures VHDL Tokens that begin with `integer` production rule: 
    /// decimal literal, based_literal, and bit_string_literals.
    /// 
    /// Assumes the incoming char `c0` was last char consumed as it a digit `0..=9`.
    fn consume_numeric(train: &mut TrainCar<impl Iterator<Item=char>>, c0: char) -> Result<VHDLToken, VHDLTokenError> {
        let mut based_delim: Option<char> = None;
        let mut number = Self::consume_value_pattern(train, Some(c0), char_set::is_digit)?;
        // check if the next char should be included
        if let Some(mut c) = train.peek() {
            // * decimal_literal
            if c == &char_set::DOT {
                number.push(train.consume().unwrap());
                // gather more integers (must exist)
                let fraction = Self::consume_value_pattern(train, None, char_set::is_digit)?;
                if fraction.is_empty() {
                    return Err(VHDLTokenError::Any(String::from("cannot have trailing decimal point")))
                // append to number
                } else {
                    number.push_str(&fraction);
                }
                // update c if there is another token to grab!
                c = if let Some(c_next) = train.peek() {
                    c_next
                } else {
                    return Ok(VHDLToken::AbstLiteral(AbstLiteral::Decimal(number)))
                };
            // * based_literal (can begin with '#' or ':')
            } else if c == &char_set::HASH || c == &char_set::COLON {
                // verify 2 <= number <= 16
                let base = interpret_integer(&number);
                if base < 2 || base > 16 {
                    return Err(VHDLTokenError::Any(String::from("based literal must have base of at least 2 and at most 16")))
                }
                based_delim = Some(*c);
                number.push(train.consume().unwrap());
                // gather initial extended digits
                // select the `eval` fn to evaluate digits
                let eval = based_integer::as_fn(base);
                let base_integers = Self::consume_value_pattern(train, None, eval)?;

                number.push_str(&base_integers);
                // stil expecting another token
                if let Some(c_next) = train.consume() {
                    // closing with a '#' or ':'
                    if c_next == based_delim.unwrap() {
                        number.push(c_next);
                    // is there a dot?
                    } else if c_next == char_set::DOT {
                        number.push(c_next);
                        // gather more integers (must exist)
                        let fraction = Self::consume_value_pattern(train, None, eval)?;
                        number.push_str(&fraction);
                        // make sure there is a closing character '#' or ':'
                        if let Some(c_next_next) = train.consume() {
                            // did not find the closing character '#' or ':'
                            if c_next_next != based_delim.unwrap() {
                                return Err(VHDLTokenError::Any(String::from("expecting closing '#' but found something else")))
                            }
                            if fraction.is_empty() {
                                return Err(VHDLTokenError::Any(String::from("expecting an integer after the dot")))
                            }
                            number.push(c_next_next);
                        // there is no more characters left to consume
                        } else {
                            if fraction.is_empty() {
                                return Err(VHDLTokenError::Any(String::from("expecting an integer after the dot")))
                            }
                            return Err(VHDLTokenError::Any(String::from("expecting closing '#'")))
                        }
                    // an unknown character
                    } else {
                        return Err(VHDLTokenError::Any(String::from("expecting closing '#' but got something else")))
                    }
                    // update c if there is another token to grab!
                    c = if let Some(c_next_next) = train.peek() {
                        c_next_next
                    } else {
                        return Ok(VHDLToken::AbstLiteral(AbstLiteral::Based(number)))
                    }
                // there is no more characters to consume
                } else {
                    return Err(VHDLTokenError::Any(String::from("expecting closing '#'")))
                }
            // * bit string literal
            } else if c != &'e' && c != &'E' && char_set::is_letter(&c) {
                // gather letters
                let mut base_spec = String::from(train.consume().unwrap());
                while let Some(c_next) = train.peek() {
                    if char_set::is_letter(c_next) == true {
                        base_spec.push(train.consume().unwrap());
                    } else {
                        break;
                    }
                }
                // verify valid base specifier
                BaseSpec::from_str(&base_spec)?;
                // force double quote to be next
                if train.peek().is_none() || train.peek().unwrap() != &char_set::DOUBLE_QUOTE {
                    return Err(VHDLTokenError::Any(String::from("expecting opening quote character for bit string literal")))
                }
                // append base_specifier
                number.push_str(&base_spec);
                // append first double quote " char
                number.push(train.consume().unwrap());
                // complete tokenizing the bit string literal
                return Ok(Self::consume_bit_str_literal(train, number)?)
            }
            // gather exponent
            if c == &'e' || c == &'E' {
                let c0 = train.consume().unwrap();
                let expon = Self::consume_exponent(train, c0)?;
                number.push_str(&expon);
            }
            return Ok(VHDLToken::AbstLiteral(match based_delim {
                Some(_) => AbstLiteral::Based(number),
                None => AbstLiteral::Decimal(number),
            }))
        } else {
            Ok(VHDLToken::AbstLiteral(AbstLiteral::Decimal(number)))
        }
    }

    /// Captures VHDL Tokens: keywords, basic identifiers, and regular bit string literals.
    /// 
    /// Assumes the first `letter` char was the last char consumed before the function call.
    fn consume_word(train: &mut TrainCar<impl Iterator<Item=char>>, c0: char) -> Result<VHDLToken, VHDLTokenError> {
        let mut word = Self::consume_value_pattern(train, Some(c0), char_set::is_letter_or_digit)?;
        match Keyword::match_keyword(&word) {
            Some(kw) => Ok(VHDLToken::Keyword(kw)),
            None => {
                // * bit string literal: check if the next char is a double quote
                if let Some(c) = train.peek() {
                    if c == &char_set::DOUBLE_QUOTE {
                        // verify valid base specifier
                        BaseSpec::from_str(&word)?;
                        // add the opening '"' character to the literal
                        word.push(train.consume().unwrap());
                        return Ok(Self::consume_bit_str_literal(train, word)?)
                    }
                }
                Ok(VHDLToken::Identifier(Identifier::Basic(word)))
            }
        }
    }

    /// Captures the remaining characters for a bit string literal.
    /// 
    /// Assumes the integer, base_specifier, and first " char are already consumed
    /// and moved as `s0`.  Rules taken from VHDL-2019 LRM p177 due to backward-compatible additions. Note
    /// that a bit string literal is allowed to have no characters within the " ".
    /// - bit_string_literal ::= \[ integer ] base_specifier " \[ bit_value ] "
    /// - bit_value ::= graphic_character { [ underline ] graphic_character } 
    fn consume_bit_str_literal(train: &mut TrainCar<impl Iterator<Item=char>>, s0: String) -> Result<VHDLToken, VHDLTokenError> {
        let mut literal = s0;
        // consume bit_value (all graphic characters except the double quote " char)
        let bit_value = Self::consume_value_pattern(train, None, char_set::is_graphic_and_not_double_quote)?;
        // verify the next character is the closing double quote " char
        if train.peek().is_none() || train.peek().unwrap() != &char_set::DOUBLE_QUOTE {
            return Err(VHDLTokenError::Any(String::from("expecting closing double quote for bit string literal")))
        }
        literal.push_str(&bit_value);
        // accept the closing " char
        literal.push(train.consume().unwrap());
        Ok(VHDLToken::BitStrLiteral(BitStrLiteral(literal)))
    }

    /// Captures an extended identifier token.
    /// 
    /// Errors if the identifier is empty.
    fn consume_extended_identifier(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, VHDLTokenError> { 
        let id = Self::consume_literal(train, &char_set::BACKSLASH)?;
        if id.is_empty() { 
            Err(VHDLTokenError::Any(String::from("extended identifier cannot be empty")))
        } else {
            Ok(VHDLToken::Identifier(Identifier::Extended(id)))
        }
    }

    /// Captures a character literal according to VHDL-2018 LRM p231. 
    /// 
    /// Assumes the first single quote '\'' was the last char consumed.
    fn consume_char_lit(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, VHDLTokenError> {
        let mut char_lit = String::with_capacity(1);
        if let Some(c) = train.consume() {
            // verify the character is a graphic character
            if char_set::is_graphic(&c) == false { return Err(VHDLTokenError::Any(String::from("char not graphic"))) }
            // add to the struct
            char_lit.push(c);
            // expect a closing single-quote @TODO handle attribute case name'attribute
            if let Some(c) = train.consume() {
                // return 
                if c != char_set::SINGLE_QUOTE {
                    return Err(VHDLTokenError::Any(String::from("expecting a single quote but got something else")))
                }
            } else {
                return Err(VHDLTokenError::Any(String::from("expecting a single quote but got none")))
            }
        }
        Ok(VHDLToken::CharLiteral(Character(char_lit)))
    }

    /// Captures a string literal.
    /// 
    /// Assumes the first double quote '\"' was the last char consumed before entering the function.
    fn consume_str_lit(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, VHDLTokenError> {
        let value = Self::consume_literal(train, &char_set::DOUBLE_QUOTE)?;
        Ok(VHDLToken::StrLiteral(value))
    }

    /// Collects a delimited comment (all characters after a `/*` up until `*/`).
    /// 
    /// Assumes the opening '/' char was the last char consumed before entering the function.
    /// Also assumes the next char is '*'.
    fn consume_delim_comment(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, VHDLTokenError> {
        // skip over opening '*'
        train.consume().expect("assumes '*' exists");
        let mut note = String::new();
        while let Some(c) = train.consume() {
            // check if we are breaking from the comment
            if c == char_set::STAR {
                if let Some(c_next) = train.peek() {
                    // break from the comment
                    if c_next == &char_set::FWDSLASH {
                        train.consume();
                        return Ok(VHDLToken::Comment(Comment::Delimited(note)))
                    }
                }
            }
            note.push(c);
        }
        Err(VHDLTokenError::Any(String::from("missing closing delimiter */")))
    }

    /// Collects a single-line comment (all characters after a `--` up until end-of-line).
    /// 
    /// Assumes the opening '-' was the last char consumed before entering the function.
    /// Also assumes the next char is '-'.
    fn consume_comment(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, VHDLTokenError> { 
        // skip over second '-'
        train.consume(); 
        // consume characters to form the comment
        let mut note = String::new();
        while let Some(c) = train.consume() {
            // cannot be vt, cr (\r), lf (\n)
            if c == '\u{000B}' || c == '\u{000D}' || c == '\u{000A}' {
                break;
            } else {
                note.push(c);
            }
        }
        Ok(VHDLToken::Comment(Comment::Single(note)))
    }

    /// Walks through the possible interpretations for capturing a VHDL delimiter.
    /// 
    /// If it successfully finds a valid VHDL delimiter, it will move the `loc` the number
    /// of characters it consumed.
    fn collect_delimiter(train: &mut TrainCar<impl Iterator<Item=char>>, c0: Option<char>) -> Result<VHDLToken, VHDLTokenError> {
        // delimiter will have at most 3 characters
        let mut delim = String::with_capacity(3);
        if let Some(c) = c0 { delim.push(c); };
        // check the next character in the sequence
        while let Some(c) = train.peek() {
            match delim.len() {
                0 => match c {
                    // ambiguous characters...read another character (could be a len-2 delimiter)
                    '?' | '<' | '>' | '/' | '=' | '*' | ':' => delim.push(train.consume().unwrap()),
                    // if it was a delimiter, take the character and increment the location
                    _ => return Self::match_delimiter(&String::from(train.consume().unwrap())),
                }
                1 => match delim.chars().nth(0).unwrap() {
                    '?' => match c {
                            // move on to next round (could be a len-3 delimiter)
                            '/' | '<' | '>' => delim.push(train.consume().unwrap()),
                            _ => return Ok(Self::match_delimiter(&delim).expect("invalid token")),
                        },
                    '<' => match c {
                            // move on to next round (could be a len-3 delimiter)
                            '=' => delim.push(train.consume().unwrap()),
                            _ => return Ok(Self::match_delimiter(&delim).expect("invalid token")),
                        },
                    _ => {
                        // try with 2
                        delim.push(*c);
                        if let Ok(op) = Self::match_delimiter(&delim) {
                            train.consume();
                            return Ok(op)
                        } else {
                            // revert back to 1
                            delim.pop();
                            return Self::match_delimiter(&delim)
                        }
                    }
                }
                2 => {
                    // try with 3
                    delim.push(*c);
                    if let Ok(op) = Self::match_delimiter(&delim) {
                        train.consume();
                        return Ok(op)
                    } else {
                        // revert back to 2 (guaranteed to exist)
                        delim.pop();
                        return Ok(Self::match_delimiter(&delim).expect("invalid token"))
                    }
                }
                _ => panic!("delimiter matching exceeds 3 characters")
            }
        };
        // try when hiting end of stream
        Self::match_delimiter(&delim)
    }

    /// Captures the generic pattern production rule by passing a fn as `eval` to compare.
    /// 
    /// This function allows for an empty result to be returned as `Ok`.
    /// - A ::= A { \[ underline ] A }
    fn consume_value_pattern(train: &mut TrainCar<impl Iterator<Item=char>>, c0: Option<char>, eval: fn(&char) -> bool) -> Result<String, VHDLTokenError> {
        let mut car = if let Some(c) = c0 { String::from(c) } else { String::new() };
        while let Some(c) = train.peek() {
            if eval(&c) == true {
                car.push(train.consume().unwrap());
            } else if c == &char_set::UNDERLINE {
                if car.is_empty() == true { return Err(VHDLTokenError::Any(String::from("expecting a digit before underline"))) }
                car.push(train.consume().unwrap());
                // a digit must proceed the underline
                if let Some(c_next) = train.consume() {
                    if eval(&c_next) == false {
                        return Err(VHDLTokenError::Any(String::from("expecting a digit to follow underline")))
                    } else {
                        car.push(c_next);
                    }
                } else {
                    return Err(VHDLTokenError::Any(String::from("expecting a digit")))
                }
            } else {
                break;
            }
        }
        Ok(car)
    }

    /// Captures an exponent.   
    ///
    /// Assumes the previous function has already peeked and determined the next char is 'E' as `c0`.
    /// - exponent ::= E \[ + ] integer | E – integer  
    fn consume_exponent(train: &mut TrainCar<impl Iterator<Item=char>>, c0: char) -> Result<String, VHDLTokenError> {
        // start with 'E'
        let mut expon = String::from(c0);
        // check for sign
        let sign = if let Some(c1) = train.consume() {
            if c1 != char_set::PLUS && c1 != char_set::DASH && char_set::is_digit(&c1) == false {
                return Err(VHDLTokenError::Any(String::from("expecting +, -, or a digit")))
            } else {
                c1
            }
        } else {
            return Err(VHDLTokenError::Any(String::from("expecting +, -, or digit but got nothing")))
        };
        // determine if c0 was a digit 
        let c0 = if char_set::is_digit(&sign) == true {
            Some(sign)
        } else {
            // add the sign to the exponent
            expon.push(sign);
            None
        };
        let value = Self::consume_value_pattern(train, c0, char_set::is_digit)?;
        if value.is_empty() {
            Err(VHDLTokenError::Any(String::from("expecting an integer exponent value but got nothing")))
        } else {
            expon.push_str(&value);
            Ok(expon)
        }
    }

    /// Walks through the stream to gather a `String` literal until finding the 
    /// exiting character `br`.
    /// 
    /// An escape is allowed by double placing the `br`, i.e. """hello"" world".
    /// Assumes the first token to parse in the stream is not the `br` character.
    /// Allows for zero or more characters in result and chars must be graphic.
    fn consume_literal(train: &mut TrainCar<impl Iterator<Item=char>>, br: &char) -> Result<String, VHDLTokenError> { 
        let mut result = String::new();
        while let Some(c) = train.consume() {
            // verify it is a graphic character
            if char_set::is_graphic(&c) == false { return Err(VHDLTokenError::Any(String::from("invalid character in literal"))) }
            // detect escape sequence
            if br == &c {
                match train.peek() {
                    Some(c_next) => if br == c_next {
                        train.consume(); // skip over escape character
                    } else {
                        return Ok(result);
                    }
                    None => return Ok(result),
                }
            } 
            result.push(c);
        }
        Err(VHDLTokenError::Any(String::from("expecting closing delimiter")))
    }
}

mod based_integer {
    /// Transforms the base `n` into its character validiation function.
    /// 
    /// The output is used to verify extended digits in a VHDL based_literal token.
    pub fn as_fn(n: usize) -> fn(c: &char) -> bool {
        match n {
            2  => is_base_2,
            3  => is_base_3,
            4  => is_base_4,
            5  => is_base_5,
            6  => is_base_6,
            7  => is_base_7,
            8  => is_base_8,
            9  => is_base_9,
            10 => is_base_10,
            11 => is_base_11,
            12 => is_base_12,
            13 => is_base_13,
            14 => is_base_14,
            15 => is_base_15,
            16 => is_base_16,
            _ => panic!("base `n` must be at least 2 and at most 16")
        }
    }

    pub fn is_base_2(c: &char)  -> bool { match c { '0'..='1' => true, _ => false, } }
    pub fn is_base_3(c: &char)  -> bool { match c { '0'..='2' => true, _ => false, } }
    pub fn is_base_4(c: &char)  -> bool { match c { '0'..='3' => true, _ => false, } }
    pub fn is_base_5(c: &char)  -> bool { match c { '0'..='4' => true, _ => false, } }
    pub fn is_base_6(c: &char)  -> bool { match c { '0'..='5' => true, _ => false, } }
    pub fn is_base_7(c: &char)  -> bool { match c { '0'..='6' => true, _ => false, } }
    pub fn is_base_8(c: &char)  -> bool { match c { '0'..='7' => true, _ => false, } }
    pub fn is_base_9(c: &char)  -> bool { match c { '0'..='8' => true, _ => false, } }
    pub fn is_base_10(c: &char) -> bool { match c { '0'..='9' => true, _ => false, } }
    pub fn is_base_11(c: &char) -> bool { match c { '0'..='9' | 'a'..='a' | 'A'..='A' => true, _ => false, } }
    pub fn is_base_12(c: &char) -> bool { match c { '0'..='9' | 'a'..='b' | 'A'..='B' => true, _ => false, } }
    pub fn is_base_13(c: &char) -> bool { match c { '0'..='9' | 'a'..='c' | 'A'..='C' => true, _ => false, } }
    pub fn is_base_14(c: &char) -> bool { match c { '0'..='9' | 'a'..='d' | 'A'..='D' => true, _ => false, } }
    pub fn is_base_15(c: &char) -> bool { match c { '0'..='9' | 'a'..='e' | 'A'..='E' => true, _ => false, } }
    pub fn is_base_16(c: &char) -> bool { match c { '0'..='9' | 'a'..='f' | 'A'..='F' => true, _ => false, } }
}

mod char_set {
    pub const DOUBLE_QUOTE: char = '\"';
    pub const BACKSLASH: char = '\\';
    pub const STAR: char = '*';
    pub const DASH: char = '-';
    pub const FWDSLASH: char = '/';
    pub const UNDERLINE: char = '_';
    pub const SINGLE_QUOTE: char = '\'';
    pub const DOT: char = '.';
    pub const HASH: char = '#';
    pub const COLON: char = ':';
    pub const PLUS: char = '+';

    /// Checks if `c` is a space according to VHDL-2008 LRM p225.
    /// Set: space, nbsp
    pub fn is_space(c: &char) -> bool {
        c == &'\u{0020}' || c == &'\u{00A0}'
    }

    /// Checks if `c` is a digit according to VHDL-2008 LRM p225.
    pub fn is_digit(c: &char) -> bool {
        match c {
            '0'..='9' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a graphic character according to VHDL-2008 LRM p230.
    /// - rule ::= upper_case_letter | digit | special_character | space_character 
    /// | lower_case_letter | other_special_character
    pub fn is_graphic(c: &char) -> bool {
        is_lower(&c) || is_upper(&c) || is_digit(&c) || 
        is_special(&c) || is_other_special(&c) || is_space(&c)
    }

    /// Checks if `c` is an upper-case letter according to VHDL-2019 LRM p257.
    /// Set: `ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ`
    pub fn is_upper(c: &char) -> bool {
        match c {
            '\u{00D7}' => false, // reject multiplication sign
            'A'..='Z' | 'À'..='Þ' => true,
            _ => false   
        }
    }

    /// Checks if `c` is a new-line character.
    pub fn is_newline(c: &char) -> bool {
        c == &'\n'
    }

    /// Checks if `c` is a special character according to VHDL-2008 LRM p225.
    /// Set: `"#&'()*+,-./:;<=>?@[]_`|`
    pub fn is_special(c: &char) -> bool {
        match c {
            '"' | '#' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | 
            ':' | ';' | '<'  | '=' | '>' | '?' | '@' | '[' | ']' | '_' | '`' | '|' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a graphic character according to VHDL-2008 LRM p225 and
    /// is NOT a double character ".
    /// 
    /// This function is exclusively used in the logic for collecting a bit string literal.
    pub fn is_graphic_and_not_double_quote(c: &char) -> bool {
        c != &DOUBLE_QUOTE && is_graphic(&c)
    }

    /// Checks if `c` is an "other special character" according to VHDL-2008 LRM p225.
    /// Set: `!$%\^{} ~¡¢£¤¥¦§ ̈©a«¬® ̄°±23 ́μ¶· ̧1o»1⁄41⁄23⁄4¿×÷-`
    pub fn is_other_special(c: &char) -> bool {
        match c {
            '!' | '$' | '%' | '\\' | '^' | '{' | '}' | ' ' | '~' | '-' | 
            '\u{00A1}'..='\u{00BF}' | '\u{00D7}' | '\u{00F7}' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a lower-case letter according to VHDL-2019 LRM p257.
    /// Set: `abcdefghijklmnopqrstuvwxyzßàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ`
    pub fn is_lower(c: &char) -> bool {
        match c {
            '\u{00F7}' => false, // reject division sign
            'a'..='z' | 'ß'..='ÿ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a letter according to VHDL-2019 LRM p257.
    pub fn is_letter(c: &char) -> bool {
        is_lower(&c) || is_upper(&c)
    }

    /// Checks if `c` is a digit | letter according to VHDL-2008 LRM p230.
    pub fn is_extended_digit(c: &char) -> bool {
        is_digit(&c) || is_letter(&c)
    }

    /// Checks if `c` is a digit | letter according to VHDL-2008 LRM p229.
    pub fn is_letter_or_digit(c: &char) -> bool {
        is_digit(&c) || is_letter(&c)
    }

    /// Checks if the character is a seperator according to VHDL-2019 LRM p259.
    pub fn is_separator(c: &char) -> bool {
        // whitespace: space, nbsp
        c == &'\u{0020}' || c == &'\u{00A0}' ||
        // format-effectors: ht (\t), vt, cr (\r), lf (\n)
        c == &'\u{0009}' || c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}'
    }
}

/// Set: B | O | X | UB | UO | UX | SB | SO | SX | D
#[derive(Debug, PartialEq)]
enum BaseSpec {
    B, O, X, UB, UO, UX, SB, SO, SX, D
}

impl FromStr for BaseSpec {
    type Err = VHDLTokenError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "b" | "B"  => Self::B,
            "o" | "O"  => Self::O,
            "x" | "X"  => Self::X,
            "ub" | "uB" | "Ub" | "UB" => Self::UB,
            "uo" | "uO" | "Uo" | "UO" => Self::UO,
            "ux" | "uX" | "Ux" | "UX" => Self::UX,
            "sb" | "sB" | "Sb" | "SB" => Self::SB,
            "so" | "sO" | "So" | "SO" => Self::SO,
            "sx" | "sX" | "Sx" | "SX" => Self::SX,
            "d" | "D"  => Self::D,
            _ => return Err(Self::Err::Any(format!("invalid base specifier '{}'", s)))
        })
    }
}

impl BaseSpec {
    fn as_str(&self) -> &str {
        match self {
            Self::B => "b",
            Self::O => "o",
            Self::X => "x",
            Self::UB => "ub",
            Self::UO => "uo",
            Self::UX => "ux",
            Self::SB => "sb",
            Self::SO => "so",
            Self::SX => "sx",
            Self::D => "d",
        }
    }
}

#[derive(Debug, PartialEq)]
struct VHDLElement(Result<lexer::Token<VHDLToken>, lexer::TokenError<VHDLTokenError>>);

#[derive(PartialEq)]
pub struct VHDLTokenizer {
    tokens: Vec<VHDLElement>,
}

impl VHDLTokenizer {
    /// Creates a new `VHDLTokenizer` struct.
    pub fn new() -> Self {
        Self { tokens: Vec::new(), }
    }

    /// Generates a `VHDLTokenizer` struct from source code `s`.
    /// 
    /// @TODO If `skip_err` is true, it will silently omit erroneous parsing from the
    /// final vector and guarantee to be `Ok`.
    pub fn from_source_code(s: &str) -> Self {
        Self { tokens: Self::tokenize(s).into_iter().map(|f| VHDLElement(f) ).collect() }
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    pub fn into_tokens(self) -> Vec<lexer::Token<VHDLToken>> {
        self.tokens.into_iter().filter_map(|f| {
            match f.0 {
                Ok(t) => {
                    // skip comments
                    if let &VHDLToken::Comment(_) = t.as_ref() {
                        None
                    } else {
                        Some(t)
                    }
                }
                Err(_) => None,
            }
        } ).collect()
    }
}

impl VHDLToken {
    /// Checks if the element is a particular keyword `kw`.
    pub fn check_keyword(&self, kw: &Keyword) -> bool {
        match self {
            VHDLToken::Keyword(r) => r == kw,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            VHDLToken::EOF => true,
            _ => false,
        }
    }

    /// Accesses the underlying `Identifier`, if one exists.
    pub fn get_identifier(&self) -> Option<&Identifier> {
        match self {
            VHDLToken::Identifier(id) => Some(id),
            _ => None,
        }
    }

    /// Checks if the element is a particular delimiter `d`.
    pub fn check_delimiter(&self, d: &Delimiter) -> bool {
        match self {
            VHDLToken::Delimiter(r) => r == d,
            _ => false,
        }
    }

    pub fn get_comment(&self) -> Option<&Comment> {
        match self {
            VHDLToken::Comment(r) => Some(r),
            _ => None,
        }
    }
}

impl std::fmt::Debug for VHDLTokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tk in &self.tokens {
            write!(f, "{}\t{:?}\n", tk.0.as_ref().unwrap().locate(), tk.0.as_ref().unwrap())?
        }
        Ok(())
    } 
}

impl Tokenize for VHDLTokenizer {
    type TokenType = VHDLToken;
    type Err = VHDLTokenError;

    fn tokenize(s: &str) -> Vec<Result<lexer::Token<Self::TokenType>, lexer::TokenError<Self::Err>>> {
        use lexer::{Token, TokenError};

        let mut train = TrainCar::new(s.chars());
        // store results here as we consume the characters
        let mut tokens: Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>> = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = train.consume() {
            // skip over whitespace
            if char_set::is_separator(&c) { continue; }
            let tk_loc = train.locate().clone();
            // build a token
            tokens.push(
            if char_set::is_letter(&c) {
                // collect general identifier
                match Self::TokenType::consume_word(&mut train, c) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            } else if c == char_set::BACKSLASH {
                // collect extended identifier
                match Self::TokenType::consume_extended_identifier(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            } else if c == char_set::DOUBLE_QUOTE {
                // collect string literal
                match Self::TokenType::consume_str_lit(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            } else if c == char_set::SINGLE_QUOTE && tokens.last().is_some() && tokens.last().unwrap().as_ref().is_ok() && tokens.last().unwrap().as_ref().unwrap().as_ref().is_delimiter() {
                // collect character literal
                match Self::TokenType::consume_char_lit(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            } else if char_set::is_digit(&c) {
                // collect decimal literal (or bit string literal or based literal)
                match Self::TokenType::consume_numeric(&mut train, c) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            } else if c == char_set::DASH && train.peek().is_some() && train.peek().unwrap() == &char_set::DASH {    
                // collect a single-line comment           
                match Self::TokenType::consume_comment(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            } else if c == char_set::FWDSLASH && train.peek().is_some() && train.peek().unwrap() == &char_set::STAR {
                // collect delimited (multi-line) comment
                match Self::TokenType::consume_delim_comment(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => {
                        let mut tk_loc = train.locate().clone();
                        tk_loc.next_col();// +1 col for correct alignment
                        Err(TokenError::new(e, tk_loc)) 
                    }
                }
            } else {
                // collect delimiter
                match Self::TokenType::collect_delimiter(&mut train, Some(c)) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone()))
                }
            });
        }
        // push final EOF token
        let mut tk_loc = train.locate().clone();
        tk_loc.next_col();
        tokens.push(Ok(Token::new(VHDLToken::EOF,  tk_loc)));
        tokens
    }
}

#[derive(Debug, PartialEq)]
pub enum VHDLTokenError {
    Any(String),
    Invalid(String),
    MissingAndEmpty(char),
    MissingClosingAndGot(char, char),
}

impl Display for VHDLTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Any(s) => s.to_string(),
            Self::Invalid(c) => format!("invalid character '{}' ", c),
            _ => todo!("write error message!")
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::lexer::*;

    #[test]
    fn iden_from_str() {
        let iden = "top_level";
        assert_eq!(Identifier::from_str(&iden).unwrap(), Identifier::Basic("top_level".to_owned()));

        let iden = "\\Top_LEVEL\\";
        assert_eq!(Identifier::from_str(&iden).unwrap(), Identifier::Extended("Top_LEVEL".to_owned()));

        // extra characters after closing
        let iden = "\\Top_\\LEVEL\\";
        assert_eq!(Identifier::from_str(&iden).is_err(), true);
    }

    #[test]
    fn interpret_int() {
        let contents = "16";
        assert_eq!(interpret_integer(&contents), 16);

        let contents = "1_6";
        assert_eq!(interpret_integer(&contents), 16);

        let contents = "50_000_000";
        assert_eq!(interpret_integer(&contents), 50_000_000);
    }

    #[test]
    #[should_panic]
    fn interpret_int_with_other_chars() {
        let contents = "16a";
        interpret_integer(&contents);
    }

    #[test]
    #[should_panic]
    fn interpret_int_with_no_leading_digit() {
        let contents = "";
        interpret_integer(&contents);
    }

    #[test]
    fn single_quote_as_delimiter() {
        let contents = "\
foo <= std_logic_vector'('a','b','c');";
        let tokens: Vec<VHDLToken> = VHDLTokenizer::tokenize(&contents)
            .into_iter()
            .map(|f| { f.unwrap().take() })
            .collect();
        assert_eq!(tokens, vec![
            VHDLToken::Identifier(Identifier::Basic("foo".to_owned())),
            VHDLToken::Delimiter(Delimiter::SigAssign),
            VHDLToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
            VHDLToken::Delimiter(Delimiter::SingleQuote),
            VHDLToken::Delimiter(Delimiter::ParenL),
            VHDLToken::CharLiteral(Character("a".to_owned())),
            VHDLToken::Delimiter(Delimiter::Comma),
            VHDLToken::CharLiteral(Character("b".to_owned())),
            VHDLToken::Delimiter(Delimiter::Comma),
            VHDLToken::CharLiteral(Character("c".to_owned())),
            VHDLToken::Delimiter(Delimiter::ParenR),
            VHDLToken::Delimiter(Delimiter::Terminator),
            VHDLToken::EOF,
        ]);

        let contents = "\
(clk'event = '1')";
            let tokens: Vec<VHDLToken> = VHDLTokenizer::tokenize(&contents)
            .into_iter()
            .map(|f| { f.unwrap().take() })
            .collect();
        assert_eq!(tokens, vec![
            VHDLToken::Delimiter(Delimiter::ParenL),
            VHDLToken::Identifier(Identifier::Basic("clk".to_owned())),
            VHDLToken::Delimiter(Delimiter::SingleQuote),
            VHDLToken::Identifier(Identifier::Basic("event".to_owned())),
            VHDLToken::Delimiter(Delimiter::Eq),
            VHDLToken::CharLiteral(Character("1".to_owned())),
            VHDLToken::Delimiter(Delimiter::ParenR),
            VHDLToken::EOF,
        ]);
    }

    #[test]
    fn lex_partial_bit_str() {
        let words = "b\"1010\"more text";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_word(&mut tc, c0), Ok(VHDLToken::BitStrLiteral(BitStrLiteral("b\"1010\"".to_owned()))));
        assert_eq!(tc.peekable().clone().collect::<String>(), "more text");
        assert_eq!(tc.locate(), &Position::place(1, 7));

        // invalid base specifier in any language standard
        let words = "z\"1010\"more text";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_word(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_full_bit_str() {
        let contents = "10b\"10_1001_1111\";";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::BitStrLiteral(BitStrLiteral("10b\"10_1001_1111\"".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 17));

        let contents = "12SX\"F-\";";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::BitStrLiteral(BitStrLiteral("12SX\"F-\"".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 8));
    }

    #[test]
    fn lex_numeric() {
        let contents = "32)";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("32".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), ")");

        let contents = "32_000;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("32_000".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");

        let contents = "0.456";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("0.456".to_owned())));

        let contents = "6.023E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("6.023E+24".to_owned())));

        let contents = "7#6.023#E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("7#6.023#E+24".to_owned())));

        let contents = "16#F.FF#E+2";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("16#F.FF#E+2".to_owned())));

        let contents = "2#1.1111_1111_111#E11";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("2#1.1111_1111_111#E11".to_owned())));

        let contents = "016#0FF#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("016#0FF#".to_owned())));

        let contents = "1_6#1E.1f1# -- comment";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("1_6#1E.1f1#".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), " -- comment");

        // '#' can be replaced by ':' if done in both occurences - VHDL-1993 LRM p180
        let contents = "016:0FF:";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("016:0FF:".to_owned())));

        let contents = "016:0FF#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
    }

    #[test] 
    fn based_literal_base_out_of_range() {
        let contents = "1#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "#0123456789AaBbCcDdEeFfGg#");

        let contents = "17#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "#0123456789AaBbCcDdEeFfGg#");
    }

    #[test]
    fn based_literal_digit_out_of_range() {
        let contents = "2#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "3456789AaBbCcDdEeFfGg#");

        let contents = "9#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "AaBbCcDdEeFfGg#");

        let contents = "1_0#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "aBbCcDdEeFfGg#");

        let contents = "11#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "bCcDdEeFfGg#");

        let contents = "16#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "g#");
    }

    #[test]
    fn lex_single_comment() {
        let contents = "\
--here is a vhdl comment";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume(); // already determined first dash
        assert_eq!(VHDLToken::consume_comment(&mut tc).unwrap(), VHDLToken::Comment(Comment::Single("here is a vhdl comment".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 24));

        let contents = "\
--here is a vhdl comment
entity fa is end entity;";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume(); // already determined first dash
        assert_eq!(VHDLToken::consume_comment(&mut tc).unwrap(), VHDLToken::Comment(Comment::Single("here is a vhdl comment".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), "entity fa is end entity;");
        assert_eq!(tc.locate(), &Position::place(2, 0));
    }

    #[test]
    fn lex_delim_comment() {
        let contents = "\
/* here is a vhdl 
delimited-line comment. Look at all the space! */;";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume();
        assert_eq!(VHDLToken::consume_delim_comment(&mut tc).unwrap(), VHDLToken::Comment(Comment::Delimited(" here is a vhdl 
delimited-line comment. Look at all the space! ".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(2, 49));

        let contents = "/* here is a vhdl comment";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume();
        assert_eq!(VHDLToken::consume_delim_comment(&mut tc).is_err(), true);
    }

    #[test]
    fn lex_char_literal() {
        let contents = "1'";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_char_lit(&mut tc).unwrap(), VHDLToken::CharLiteral(Character("1".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 2));

        let contents = "12'";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_char_lit(&mut tc).is_err(), true);
    }

    #[test]
    fn lex_expon() {
        let contents = "E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_exponent(&mut tc, c0).unwrap(), "E+24");
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 4));

        let contents = "e6;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_exponent(&mut tc, c0).unwrap(), "e6");
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 2));

        let contents = "e-12;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_exponent(&mut tc, c0).unwrap(), "e-12");
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");

        // negative test cases
        let contents = "e-;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_exponent(&mut tc, c0).is_err(), true);

        let contents = "e+2_;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_exponent(&mut tc, c0).is_err(), true);

        let contents = "e";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_exponent(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_integer() {
        // allow bit string literal to be none
        let contents = "";
        // testing using digit prod. rule "graphic"
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_graphic).unwrap(), "");
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 0));

        let contents = "234";
        // testing using digit prod. rule "integer"
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_digit).unwrap(), "234");
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 3));

        let contents = "1_2_345 ";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_digit).unwrap(), "1_2_345");
        assert_eq!(tc.peekable().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position::place(1, 7));

        let contents = "23__4";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_digit).is_err(), true); // double underscore

        let contents = "_24";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_digit).is_err(), true); // leading underscore

        let contents = "_23_4";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, Some('1'), char_set::is_digit).is_ok(), true); 

        // testing using extended_digit prod. rule "based_integer"
        let contents = "abcd_FFFF_0021";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_extended_digit).unwrap(), "abcd_FFFF_0021");

        // testing using graphic_char prod. rule "bit_value"
        let contents = "XXXX_01LH_F--1";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::consume_value_pattern(&mut tc, None, char_set::is_graphic).unwrap(), "XXXX_01LH_F--1");
    }

    #[test]
    fn lex_identifier() {
        let words = "entity is";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_word(&mut tc, c0).unwrap(), VHDLToken::Keyword(Keyword::Entity));
        assert_eq!(tc.peekable().clone().collect::<String>(), " is");
        assert_eq!(tc.locate(), &Position::place(1, 6));

        let words = "std_logic_1164.all;";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_word(&mut tc, c0).unwrap(), VHDLToken::Identifier(Identifier::Basic("std_logic_1164".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), ".all;");
        assert_eq!(tc.locate(), &Position::place(1, 14));

        let words = "ready_OUT<=";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_word(&mut tc, c0).unwrap(), VHDLToken::Identifier(Identifier::Basic("ready_OUT".to_owned())));
        assert_eq!(tc.peekable().clone().collect::<String>(), "<=");
        assert_eq!(tc.locate(), &Position::place(1, 9));
    }

    #[test]
    fn lex_literal() {
        let contents = "\" go Gators! \" ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), " go Gators! ");
        assert_eq!(tc.peekable().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position::place(1, 14));

        let contents = "\" go \"\"to\"\"\" ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), " go \"to\"");
        assert_eq!(tc.peekable().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position::place(1, 12));

        let contents = "\"go ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).is_err(), true); // no closing quote
    }

    #[test]
    fn lex_literal_2() {
        let contents = "\"Setup time is too short\"more text";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), "Setup time is too short");
        assert_eq!(tc.peekable().clone().collect::<String>(), "more text");
        assert_eq!(tc.locate(), &Position::place(1, 25));

        let contents = "\"\"\"\"\"\"";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), "\"\"");
        assert_eq!(tc.locate(), &Position::place(1, 6));

        let contents = "\" go \"\"gators\"\" from UF! \"";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), " go \"gators\" from UF! ");
        assert_eq!(tc.locate(), &Position::place(1, 26));

        let contents = "\\VHDL\\";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), "VHDL");

        let contents = "\\a\\\\b\\more text afterward";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VHDLToken::consume_literal(&mut tc, &c0).unwrap(), "a\\b");
        // verify the stream is left in the correct state
        assert_eq!(tc.peekable().clone().collect::<String>(), "more text afterward");
    }

    #[test]
    fn lex_tokens() {
        let s = "\
entity fa is end entity;";
        let tokens: Vec<VHDLToken> = VHDLTokenizer::tokenize(s)
            .into_iter()
            .map(|f| { f.unwrap().take() })
            .collect();
        assert_eq!(tokens, vec![
            VHDLToken::Keyword(Keyword::Entity),
            VHDLToken::Identifier(Identifier::Basic("fa".to_owned())),
            VHDLToken::Keyword(Keyword::Is),
            VHDLToken::Keyword(Keyword::End),
            VHDLToken::Keyword(Keyword::Entity),
            VHDLToken::Delimiter(Delimiter::Terminator),
            VHDLToken::EOF,
        ]);
    }

    #[test]
    fn lex_comment_token() {
        let s = "\
-- here is a vhdl single-line comment!";
        let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s).into_iter().map(|f| f.unwrap()).collect();
        assert_eq!(tokens, vec![
            Token::new(VHDLToken::Comment(Comment::Single(" here is a vhdl single-line comment!".to_owned())), Position::place(1, 1)),
            Token::new(VHDLToken::EOF, Position::place(1, 39)),
        ]);
    }

    #[test]
    fn lex_comment_token_delim() {
        let s = "\
/* here is a vhdl 
    delimited-line comment. Look at all the space! */";
        let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s).into_iter().map(|f| f.unwrap()).collect();
        assert_eq!(tokens, vec![
            Token::new(VHDLToken::Comment(Comment::Delimited(" here is a vhdl 
    delimited-line comment. Look at all the space! ".to_owned())), Position::place(1, 1)),
            Token::new(VHDLToken::EOF, Position::place(2, 54)),
        ]);
    }

    #[test]
    fn lex_vhdl_line() {
        let s = "\
signal magic_num : std_logic := '1';";
        let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s).into_iter().map(|f| f.unwrap()).collect();
        assert_eq!(tokens, vec![
            Token::new(VHDLToken::Keyword(Keyword::Signal), Position::place(1, 1)),
            Token::new(VHDLToken::Identifier(Identifier::Basic("magic_num".to_owned())), Position::place(1, 8)),
            Token::new(VHDLToken::Delimiter(Delimiter::Colon), Position::place(1, 18)),
            Token::new(VHDLToken::Identifier(Identifier::Basic("std_logic".to_owned())), Position::place(1, 20)),
            Token::new(VHDLToken::Delimiter(Delimiter::VarAssign), Position::place(1, 30)),
            Token::new(VHDLToken::CharLiteral(Character("1".to_owned())), Position::place(1, 33)),
            Token::new(VHDLToken::Delimiter(Delimiter::Terminator), Position::place(1, 36)),
            Token::new(VHDLToken::EOF, Position::place(1, 37)),
        ]);
    }

    #[test]
    fn locate_tokens() {
        let s = "\
entity fa is end entity;";
        let tokens: Vec<Position> = VHDLTokenizer::tokenize(s)
            .into_iter()
            .map(|f| { f.unwrap().locate().clone() })
            .collect();
        assert_eq!(tokens, vec![
            Position::place(1, 1),  // 1:1 keyword: entity
            Position::place(1, 8),  // 1:8 basic identifier: fa
            Position::place(1, 11), // 1:11 keyword: is
            Position::place(1, 14), // 1:14 keyword: end
            Position::place(1, 18), // 1:18 keyword: entity
            Position::place(1, 24), // 1:24 delimiter: ;
            Position::place(1, 25), // 1:25 eof
        ]);  
    }

    #[test]
    fn lex_delimiter_single() {
        let contents = "&";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::Ampersand)));
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 1));

        let contents = "?";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::Question)));
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 1));

        let contents = "< MAX_COUNT";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::Lt)));
        assert_eq!(tc.peekable().clone().collect::<String>(), " MAX_COUNT");
        assert_eq!(tc.locate(), &Position::place(1, 1));

        let contents = ");";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::ParenR)));
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 1));
    }

    #[test]
    fn lex_delimiter_none() {
        let contents = "fa";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "a");
        assert_eq!(tc.locate(), &Position::place(1, 1));
    }

    #[test]
    fn lex_delimiter_double() {
        let contents = "<=";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::SigAssign)));
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 2));

        let contents = "**WIDTH";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::DoubleStar)));
        assert_eq!(tc.peekable().clone().collect::<String>(), "WIDTH");
        assert_eq!(tc.locate(), &Position::place(1, 2));
    }

    #[test]
    fn lex_delimiter_triple() {
        let contents = "<=>";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::SigAssoc)));
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 3));

        let contents = "?/= MAGIC_NUM";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VHDLToken::collect_delimiter(&mut tc, None), Ok(VHDLToken::Delimiter(Delimiter::MatchNE)));
        assert_eq!(tc.peekable().clone().collect::<String>(), " MAGIC_NUM");
        assert_eq!(tc.locate(), &Position::place(1, 3));
    }

    #[test]
    fn match_delimiter() {
        let word = "<=";
        assert_eq!(VHDLToken::match_delimiter(word), Ok(VHDLToken::Delimiter(Delimiter::SigAssign)));

        let word = "-";
        assert_eq!(VHDLToken::match_delimiter(word), Ok(VHDLToken::Delimiter(Delimiter::Dash)));

        let word = "<=>";
        assert_eq!(VHDLToken::match_delimiter(word), Ok(VHDLToken::Delimiter(Delimiter::SigAssoc)));

        let word = "^";
        assert_eq!(VHDLToken::match_delimiter(word).is_err(), true);

        let word = "entity";
        assert_eq!(VHDLToken::match_delimiter(word).is_err(), true);
    }

    #[test]
    fn match_reserved_idenifier() {
        let word = "END";
        assert_eq!(Keyword::match_keyword(word), Some(Keyword::End));

        let word = "EnTITY";
        assert_eq!(Keyword::match_keyword(word), Some(Keyword::Entity));

        let word = "entitys";
        assert_eq!(Keyword::match_keyword(word), None);

        let word = "<=";
        assert_eq!(Keyword::match_keyword(word), None);
    }

    #[test]
    fn is_separator() {
        let c = ' '; // space
        assert_eq!(char_set::is_separator(&c), true);

        let c = '\u{00A0}'; // nbsp
        assert_eq!(char_set::is_separator(&c), true);

        let c = '\t'; // horizontal tab
        assert_eq!(char_set::is_separator(&c), true);

        let c = '\n'; // new-line
        assert_eq!(char_set::is_separator(&c), true);

        let c = 'c';  // negative case: ascii char
        assert_eq!(char_set::is_separator(&c), false);
    }
    
    #[test]
    fn identifier_equality_and_len() {
        let id0 = Identifier::Basic("fa".to_owned());
        let id1 = Identifier::Basic("Fa".to_owned());
        assert_eq!(id1.len(), 2);
        assert_eq!(id0, id1);

        let id0 = Identifier::Basic("fa".to_owned());
        let id1 = Identifier::Basic("Full_adder".to_owned());
        assert_ne!(id0, id1);

        let id0 = Identifier::Basic("VHDL".to_owned());    // written as: VHDL
        let id1 = Identifier::Extended("VHDL".to_owned()); // written as: \VHDL\
        assert_ne!(id0, id1);

        let id0 = Identifier::Extended("vhdl".to_owned()); // written as: \vhdl\
        let id1 = Identifier::Extended("VHDL".to_owned()); // written as: \VHDL\
        assert_ne!(id0, id1);
        assert_eq!(id1.len(), 6);

        let id0 = Identifier::Extended("I\\D".to_owned()); // written as: \I\\D\
        assert_eq!(id0.len(), 6);
        
        let id0 = Identifier::from_str("\\I\\\\DEN\\").unwrap(); // written as: \I\\D\
        assert_eq!(id0.len(), 8);
    }

    #[test]
    #[ignore]
    fn playground_code() {
        let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity \\nor_gate\\ is --$ -- error on this line
    generic(
        N: positive
    );
    port(
        a : in std_logic_vector(N-1 downto 0);
        \\In\\ : in std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
end entity nor_gate;

architecture rtl of nor_gate is
    constant GO_ADDR_MMAP:integer:=2#001_1100.001#E14;
    constant freq_hz : unsigned := 50_000_000;
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx\"\"
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 0 -- 8c\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl; /* long comment */";
        let vhdl = VHDLTokenizer::from_source_code(&s);
        println!("{:?}", vhdl);
        panic!("manually inspect token list")
    }
}