//! VHDL tokenizer
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
/// (Line, Col)
struct Position(usize, usize);

impl Position {
    /// Creates a new `Position` struct as line 1, col 0.
    fn new() -> Self {
        Position(1, 0)
    }

    /// Increments the column counter by 1.
    fn next_col(&mut self) {
        self.1 += 1;
    }   

    /// Increments the column counter by 1. If the current char `c` is a newline,
    /// it will then drop down to the next line.
    fn step(&mut self, c: &char) {
        // @TODO step by +4 if encountered a tab?
        self.next_col();
        if c == &'\n' {
            self.next_line();
        }
    }

    /// Increments the line counter by 1.
    /// 
    /// Also resets the column counter to 0.
    fn next_line(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    /// Access the line (`.0`) number.
    fn line(&self) -> usize {
        self.0
    }

    /// Access the col (`.1`) number.
    fn col(&self) -> usize {
        self.1
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug, PartialEq)]
struct Token<T> {
    position: Position,
    ttype: T,
}

impl<T> Token<T> {
    /// Reveals the token type.
    fn unwrap(&self) -> &T {
        &self.ttype
    }

    /// Transforms the token into its type.
    fn take(self) -> T {
        self.ttype
    }

    /// Returns the position in the file where the token was captured.
    fn locate(&self) -> &Position {
        &self.position
    }

    /// Creates a new token.
    fn new(ttype: T, loc: Position) -> Self {
        Self {
            position: loc,
            ttype: ttype,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Comment {
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

#[derive(Debug, PartialEq)]
struct Character(String);

impl Character {
    fn new(c: char) -> Self {
        Self(String::from(c))
    }

    fn as_str(&self) -> &str {
        &self.0.as_ref()
    }
}

#[derive(Debug, PartialEq)]
struct BitStrLiteral(String);

impl BitStrLiteral {
    // Returns the reference to the inner `String` struct.
    fn as_str(&self) -> &str {
        &self.0.as_ref()
    }
}

// B|O|X|UB|UO|UX|SB|SO|SX|D
#[derive(Debug, PartialEq)]
enum BaseSpec {
    B,
    O,
    X,
    UB,
    UO,
    UX,
    SB,
    SO,
    SX,
    D
}

impl std::str::FromStr for BaseSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "b"  => Self::B,
            "o"  => Self::O,
            "x"  => Self::X,
            "ub" => Self::UB,
            "uo" => Self::UO,
            "ux" => Self::UX,
            "sb" => Self::SB,
            "so" => Self::SO,
            "sx" => Self::SX,
            "d"  => Self::D,
            _ => return Err(String::from("invalid base specifier"))
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

#[derive(Debug)]
enum Identifier {
    Basic(String),
    Extended(String),
}

impl Identifier {
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
}

impl std::cmp::Eq for Identifier {}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // instantly not equal if not they are not of same type
        if self.is_extended() != other.is_extended() { return false };
        // compare with case sensitivity
        if self.is_extended() == true {
            self.as_str() == other.as_str()
        // compare without case sensitivity
        } else {
            cmp_ignore_case(self.as_str(), other.as_str())
        }
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Extended(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AbstLiteral {
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

trait Tokenize {
    type TokenType;
    fn tokenize(s: &str) -> Vec<Token<Self::TokenType>>;
} 

#[derive(Debug, PartialEq)]
enum VHDLToken {
    Comment(Comment),               // (String) 
    Identifier(Identifier),         // (String) ...can be general or extended (case-sensitive) identifier
    AbstLiteral(AbstLiteral),       // (String)
    CharLiteral(Character),         // (char)
    StrLiteral(String),             // (String)
    BitStrLiteral(BitStrLiteral),  // (String)
    EOF,
    // --- delimiters
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
    // --- keywords
    Abs,
    Access,
    After,
    Alias,
    All,
    And,
    Architecture,
    Array,
    Assert,
    Assume,
    // AssumeGuarantee is omitted from VHDL-2019 LRM
    Attribute,
    Begin,
    Block,
    Body,
    Buffer,
    Bus,
    Case, 
    Component,
    Configuration,
    Constant, 
    Context,
    Cover,
    Default,
    Disconnect, 
    Downto,
    Else, 
    Elsif,
    End,
    Entity, 
    Exit,
    Fairness,
    File,
    For, 
    Force,
    Function,
    Generate, 
    Generic, 
    Group, 
    Guarded,
    If,
    Impure, 
    In, 
    Inertial, 
    Inout, 
    Is,
    Label, 
    Library, 
    Linkage, 
    Literal, 
    Loop,
    Map,
    Mod,
    Nand,
    New, 
    Next, 
    Nor, 
    Not, 
    Null,
    Of,
    On,
    Open,
    Or, 
    Others, 
    Out,
    Package, 
    Parameter, 
    Port, 
    Postponed, 
    Private,
    Procedure, 
    Process, 
    Property, 
    Protected, 
    Pure,
    Range,
    Record,
    Register,
    Reject,
    Release,
    Rem,
    Report,
    Restrict, 
    // RestrictGuarantee is omitted from VHDL-2019 LRM
    Return,
    Rol, 
    Ror,
    Select, 
    Sequence, 
    Severity,
    Signal, 
    Shared, 
    Sla,
    Sll,
    Sra,
    Srl, 
    Strong, 
    Subtype,
    Then,
    To, 
    Transport, 
    Type,
    Unaffected, 
    Units,
    Until,
    Use,
    Variable, 
    View,
    Vmode, 
    Vpkg,
    Vprop, 
    Vunit,
    Wait, 
    When, 
    While, 
    With,
    Xnor, 
    Xor,
}

/// Walks through the possible interpretations for capturing a VHDL delimiter.
/// 
/// If it successfully finds a valid VHDL delimiter, it will move the `loc` the number
/// of characters it consumed.
fn collect_delimiter(train: &mut TrainCar<impl Iterator<Item=char>>, c0: Option<char>) -> Result<VHDLToken, String> {
    let mut delim = String::with_capacity(3);
    if let Some(c) = c0 { delim.push(c); }

    while let Some(c) = train.peek() {
        match delim.len() {
            0 => match c {
                // ambiguous characters...read another character (could be a len-2 delimiter)
                '?' | '<' | '>' | '/' | '=' | '*' | ':' => {
                    delim.push(train.consume().unwrap())
                },
                _ => { 
                    // if it was a delimiter, take the character and increment the location
                    return VHDLToken::match_delimiter(&String::from(train.consume().unwrap()))
                }
            }
            1 => match delim.chars().nth(0).unwrap() {
                '?' => {
                    match c {
                        // move on to next round (could be a len-3 delimiter)
                        '/' | '<' | '>' => {
                            delim.push(train.consume().unwrap())
                        }
                        _ => { return Ok(VHDLToken::match_delimiter(&delim).expect("invalid token")) }
                    }
                }
                '<' => {
                    match c {
                        // move on to next round (could be a len-3 delimiter)
                        '=' => {
                            delim.push(train.consume().unwrap())
                        },
                        _ => { return Ok(VHDLToken::match_delimiter(&delim).expect("invalid token")) }
                    }
                }
                _ => {
                    // try with 2
                    delim.push(*c);
                    if let Ok(op) = VHDLToken::match_delimiter(&delim) {
                        train.consume();
                        return Ok(op)
                    } else {
                        // revert back to 1
                        delim.pop();
                        return VHDLToken::match_delimiter(&delim)
                    }
                }
            }
            2 => {
                // try with 3
                delim.push(*c);
                if let Ok(op) = VHDLToken::match_delimiter(&delim) {
                    train.consume();
                    return Ok(op)
                } else {
                    // revert back to 2 (guaranteed to exist)
                    delim.pop();
                    return Ok(VHDLToken::match_delimiter(&delim).expect("invalid token"))
                }
            }
            _ => panic!("delimiter matching exceeds 3 characters")
        }
    };
    // try when hiting end of stream
    VHDLToken::match_delimiter(&delim)
}

impl VHDLToken {
    /// Attempts to match the given string of characters `s` to a VHDL delimiter.
    fn match_delimiter(s: &str) -> Result<Self, String> {
        Ok(match s {
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
            _ => return Err(format!("invalid character: {}", s)),
        })
    }

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
            _ => return None
        })
    }
}

impl std::fmt::Display for VHDLToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Comment(note) => note.as_str(),
            Self::Identifier(id) => id.as_str(),
            Self::AbstLiteral(a) => a.as_str(),
            Self::CharLiteral(c) => c.as_str(),
            Self::StrLiteral(s) => s.as_ref(),
            Self::BitStrLiteral(b) => b.as_str(),
            Self::EOF           => "EOF",
            // --- delimiters
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
            // --- keywords
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
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq)]
struct VHDLTokenizer {
    inner: Vec<Token<VHDLToken>>,
}

impl std::fmt::Debug for VHDLTokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tk in &self.inner {
            write!(f, "{}\t{:?}\n", tk.locate(), tk.unwrap())?
        }
        Ok(())
    } 
}

/// Compares to string references `s0` and `s1` with case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding case sensivity.
fn cmp_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_lowercase().cmp(s1.next().unwrap().to_lowercase()) != std::cmp::Ordering::Equal {
            return false
        }
    }
    true
}

/// Compares to string references `s0` and `s1` with only ascii case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding ascii case sensivity.
fn cmp_ascii_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_ascii_lowercase() != s1.next().unwrap().to_ascii_lowercase() {
            return false
        }
    }
    true
}

mod char_set {
    pub const ASCII_ZERO: usize = '0' as usize;
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

/// Checks is a character `c` is within the given extended digit range set by `b`.
fn in_range(b: usize, c: &char) -> bool {
    let within_digit = (*c as usize) < char_set::ASCII_ZERO + b && (*c as usize) >= char_set::ASCII_ZERO;
    if b <= 10 {
        return within_digit
    } else {
        match b {
            11 => within_digit || match c { 'a'..='a' | 'A'..='A' => true, _ => false },
            12 => within_digit || match c { 'a'..='b' | 'A'..='B' => true, _ => false },
            13 => within_digit || match c { 'a'..='c' | 'A'..='C' => true, _ => false },
            14 => within_digit || match c { 'a'..='d' | 'A'..='D' => true, _ => false },
            15 => within_digit || match c { 'a'..='e' | 'A'..='E' => true, _ => false },
            16 => within_digit || match c { 'a'..='f' | 'A'..='F' => true, _ => false },
            _ => panic!("invalid base (only 2-16)")
        }
    }
}

impl Tokenize for VHDLTokenizer {
    type TokenType = VHDLToken;

    fn tokenize(s: &str) -> Vec<Token<Self::TokenType>> {
        let mut train = TrainCar::new(s.chars());
        // store results here as we consume the characters
        let mut tokens = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = train.consume() {

            let tk_loc = train.locate().clone();
            if char_set::is_letter(&c) {
                // collect general identifier (@TODO or bit string literal)
                let tk = consume_word(&mut train, c).unwrap(); 
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::BACKSLASH {
                // collect extended identifier
                let tk = consume_extended_identifier(&mut train).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::DOUBLE_QUOTE {
                // collect string literal
                let tk = consume_str_lit(&mut train).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::SINGLE_QUOTE {
                // collect character literal
                let tk = consume_char_lit(&mut train).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if char_set::is_digit(&c) {
                // collect decimal literal (or bit string literal or based literal)
                let tk = consume_numeric(&mut train, c).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::DASH && train.peek().is_some() && train.peek().unwrap() == &char_set::DASH {    
                // collect a single-line comment           
                let tk = consume_comment(&mut train).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::FWDSLASH && train.peek().is_some() && train.peek().unwrap() == &char_set::STAR {
                // collect delimited (multi-line) comment
                let tk = consume_delim_comment(&mut train).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if char_set::is_separator(&c) == false {
                // collect delimiter
                let tk = collect_delimiter(&mut train, Some(c)).unwrap();
                tokens.push(Token::new(tk, tk_loc));
            }
        }
        // push final EOF token
        let mut tk_loc = train.locate().clone();
        tk_loc.next_col();
        tokens.push(Token::new(VHDLToken::EOF,  tk_loc.clone()));
        tokens
    }
}


// --- REFACTORED SECTION ---

/// Captures VHDL Tokens that begin with `integer` production rule: 
/// decimal literal, based_literal, and bit_string_literals.
/// 
/// Assumes the incoming char `c0` was last char consumed as it a digit `0..=9`.
fn consume_numeric(train: &mut TrainCar<impl Iterator<Item=char>>, c0: char) -> Result<VHDLToken, String> {
    let mut based_delim: Option<char> = None;
    let mut number = consume_integer(train, Some(c0), char_set::is_digit)?;
    // check if the next char should be included
    if let Some(mut c) = train.peek() {
        // * decimal_literal
        if c == &char_set::DOT {
            number.push(train.consume().unwrap());
            // gather more integers (must exist)
            let fraction = consume_integer(train, None, char_set::is_digit)?;
            if fraction.is_empty() {
                return Err(String::from("cannot have trailing decimal point"))
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
            based_delim = Some(*c);
            number.push(train.consume().unwrap());
            // gather first base integers
            let base_integers = consume_integer(train, None, char_set::is_extended_digit)?;
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
                    let fraction = consume_integer(train, None, char_set::is_extended_digit)?;
                    if fraction.is_empty() {
                        return Err(String::from("cannot have trailing decimal point"))
                    // append to number
                    } else {
                        number.push_str(&fraction);
                    }
                    // make sure there is a closing HASH
                    if let Some(c_next_next) = train.consume() {
                        if c_next_next != based_delim.unwrap() {
                            return Err(String::from("expecting closing '#' but found something else"))
                        }
                        number.push(c_next_next);
                    } else {
                        return Err(String::from("expecting closing '#'"))
                    }
                } else {
                    return Err(String::from("expecting closing '#'"))
                }
                // update c if there is another token to grab!
                c = if let Some(c_next_next) = train.peek() {
                    c_next_next
                } else {
                    return Ok(VHDLToken::AbstLiteral(AbstLiteral::Based(number)))
                }
            } else {
                return Err(String::from("expecting closing '#'"))
            }
        // * bit string literal
        } else if c != &'e' && c != &'E' && char_set::is_letter(&c) {
            // gather letters
            let base_spec = consume_integer(train, None, char_set::is_letter)?;
            // verify valid base specifier
            BaseSpec::from_str(&base_spec)?;
            // force double quote to be next
            if train.peek().is_none() || train.peek().unwrap() != &char_set::DOUBLE_QUOTE {
                return Err(String::from("expecting opening quote character for bit string literal"))
            }
            // append base_specifier
            number.push_str(&base_spec);
            // append first double quote " char
            number.push(train.consume().unwrap());
            // complete tokenizing the bit string literal
            return Ok(consume_bit_str_literal(train, number)?)
        }
        // gather exponent
        if c == &'e' || c == &'E' {
            let c0 = train.consume().unwrap();
            let expon = consume_exponent(train, c0)?;
            number.push_str(&expon);
        }
        return Ok(VHDLToken::AbstLiteral(match based_delim {
            Some(_) => AbstLiteral::Based(number),
            None => AbstLiteral::Decimal(number),
        }))
    } else {
        Ok(VHDLToken::AbstLiteral(AbstLiteral::Decimal(number)))
    }
    // check for exponent, dot, -> decimal_literal
    // check for hash -> based_literal
    // check for alphabetic characters other than 'e' -> bit_str_literal
    // none
}

use std::str::FromStr;

/// Captures VHDL Tokens: keywords and basic identifiers.
/// 
/// Assumes the first `letter` char was the last char consumed before the function call.
fn consume_word(train: &mut TrainCar<impl Iterator<Item=char>>, c0: char) -> Result<VHDLToken, String> {
    let mut word = consume_integer(train, Some(c0), char_set::is_letter_or_digit)?;
    match VHDLToken::match_keyword(&word) {
        Some(keyword) => Ok(keyword),
        None => {
            // * bit string literal: check if the next char is a double quote
            if let Some(c) = train.peek() {
                if c == &char_set::DOUBLE_QUOTE {
                    // verify valid base specifier
                    BaseSpec::from_str(&word)?;
                    // add the opening '"' character to the literal
                    word.push(train.consume().unwrap());
                    return Ok(consume_bit_str_literal(train, word)?)
                }
            }
            Ok(VHDLToken::Identifier(Identifier::Basic(word)))
        }
    }
}

/// Captures the remaining characters for a bit string literal.
/// 
/// Assumes the integer, base_specifier, and first " char are already consumed
/// and moved as `s0`.  Rules taken from VHDL 2019 LRM p177 due to backward-compatible additions. Note
/// a bit string literal is allowed to have no characters within the " ".
/// - bit_string_literal ::= \[ integer ] base_specifier " \[ bit_value ] "
/// - bit_value ::= graphic_character { [ underline ] graphic_character } 
fn consume_bit_str_literal(train: &mut TrainCar<impl Iterator<Item=char>>, s0: String) -> Result<VHDLToken, String> {
    let mut literal = s0;
    // consume bit_value (all graphic characters except the double quote " char)
    let bit_value = consume_integer(train, None, char_set::is_graphic_and_not_double_quote)?;
    // verify the next character is the closing double quote " char
    if train.peek().is_none() || train.peek().unwrap() != &char_set::DOUBLE_QUOTE {
        return Err(String::from("expecting closing double quote for bit string literal"))
    }
    literal.push_str(&bit_value);
    // accept the closing " char
    literal.push(train.consume().unwrap());
    Ok(VHDLToken::BitStrLiteral(BitStrLiteral(literal)))
}

/// Captures the generic pattern production rule by passing a fn as `eval` to compare.
/// - A ::= A { \[ underline ] A }
fn consume_integer(train: &mut TrainCar<impl Iterator<Item=char>>, c0: Option<char>, eval: fn(&char) -> bool) -> Result<String, String> {
        let mut car = if let Some(c) = c0 { String::from(c) } else { String::new() };
        while let Some(c) = train.peek() {
            if eval(&c) == true {
                car.push(train.consume().unwrap());
            } else if c == &char_set::UNDERLINE {
                if car.is_empty() == true { return Err(String::from("expecting a digit before underline")) }
                car.push(train.consume().unwrap());
                // a digit must proceed the underline
                if let Some(c_next) = train.consume() {
                    if eval(&c_next) == false {
                        return Err(String::from("expecting a digit to follow underline"))
                    } else {
                        car.push(c_next);
                    }
                } else {
                    return Err(String::from("expecting a digit"))
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
fn consume_exponent(train: &mut TrainCar<impl Iterator<Item=char>>, c0: char) -> Result<String, String> {
    // start with 'E'
    let mut expon = String::from(c0);
    // check for sign
    let sign = if let Some(c1) = train.consume() {
        if c1 != char_set::PLUS && c1 != char_set::DASH && char_set::is_digit(&c1) == false {
            return Err(String::from("expecting +, -, or a digit"))
        } else {
            c1
        }
    } else {
        return Err(String::from("expecting +, -, or digit but got nothing"))
    };
    // determine if c0 was a digit 
    let c0 = if char_set::is_digit(&sign) == true {
        Some(sign)
    } else {
        // add the sign to the exponent
        expon.push(sign);
        None
    };
    let value = consume_integer(train, c0, char_set::is_digit)?;
    if value.is_empty() {
        Err(String::from("expecting an integer exponent value but got nothing"))
    } else {
        expon.push_str(&value);
        Ok(expon)
    }
}

/// Captures an extended identifier token.
/// 
/// Errors if the identifier is empty.
/// train: &mut TrainCar<impl Iterator<Item=char>>, c0: Option<char>) -> Result<String, String> {
fn consume_extended_identifier(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, String> { 
    let id = consume_literal(train, &char_set::BACKSLASH)?;
    if id.is_empty() { 
        Err(String::from("extended identifier cannot be empty")) 
    } else {
        Ok(VHDLToken::Identifier(Identifier::Extended(id)))
    }
}

/// Walks through the stream to gather a `String` literal until finding the 
/// exiting character `br`.
/// 
/// An escape is allowed by double placing the `br`, i.e. """hello"" world".
/// Assumes the first token to parse in the stream is not the `br` character.
/// Allows for zero or more characters in result and chars must be graphic.
fn consume_literal(train: &mut TrainCar<impl Iterator<Item=char>>, br: &char) -> Result<String, String> { 
        let mut result = String::new();
        while let Some(c) = train.consume() {
            // verify it is a graphic character
            if char_set::is_graphic(&c) == false { return Err(String::from("invalid character in literal")) }
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
        Err(String::from("expecting closing delimiter"))
}

/// Captures a character literal according to VHDL-2018 LRM p231. 
/// 
/// Assumes the first single quote '\'' was the last char consumed.
fn consume_char_lit(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, String> {
    let mut char_lit = String::with_capacity(1);
    if let Some(c) = train.consume() {
        // verify the character is a graphic character
        if char_set::is_graphic(&c) == false { return Err(String::from("char not graphic")) }
        // add to the struct
        char_lit.push(c);
        // expect a closing single-quote 
        if let Some(c) = train.consume() {
            if c != char_set::SINGLE_QUOTE {
                return Err(String::from("expecting a single quote but got something else"))
            }
        } else {
            return Err(String::from("expecting a single quote but got none"))
        }
    }
    Ok(VHDLToken::CharLiteral(Character(char_lit)))
}

/// Captures a string literal.
/// 
/// Assumes the first double quote '\"' was the last char consumed before entering the function.
fn consume_str_lit(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, String> {
    let value = consume_literal(train, &char_set::DOUBLE_QUOTE)?;
    Ok(VHDLToken::StrLiteral(value))
}

/// Collects a delimited comment (all characters after a `/*` up until `*/`).
/// 
/// Assumes the opening '/' char was the last char consumed before entering the function.
/// Also assumes the next char is '*'.
fn consume_delim_comment(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, String> {
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
    Err(String::from("missing closing delimiter */"))
}

/// Collects a single-line comment (all characters after a `--` up until end-of-line).
/// 
/// Assumes the opening '-' was the last char consumed before entering the function.
/// Also assumes the next char is '-'.
fn consume_comment(train: &mut TrainCar<impl Iterator<Item=char>>) -> Result<VHDLToken, String> { 
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

/// Helps keep the current position in the contents as the characters are consumed.
struct TrainCar<T> where T: Iterator<Item=char> {
    contents: Peekable<T>,
    loc: Position,
}

impl<T> TrainCar<T> where T: Iterator<Item=char> {
    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.contents.next() {
            self.loc.step(&c);
            Some(c)
        } else {
            None
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.contents.peek()
    }

    fn new(s: T) -> Self {
        Self {
            loc: Position::new(),
            contents: s.peekable(),
        }
    }

    fn as_ref(&self) -> &Peekable<T> {
        &self.contents
    }

    /// Access the position of the first remainig character.
    fn locate(&self) -> &Position {
        &self.loc
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lex_partial_bit_str() {
        let words = "b\"1010\"more text";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_word(&mut tc, c0), Ok(VHDLToken::BitStrLiteral(BitStrLiteral("b\"1010\"".to_owned()))));
        assert_eq!(tc.as_ref().clone().collect::<String>(), "more text");
        assert_eq!(tc.locate(), &Position(1, 7));

        // invalid base specifier in any language standard
        let words = "z\"1010\"more text";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_word(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_full_bit_str() {
        let contents = "10b\"10_1001_1111\";";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::BitStrLiteral(BitStrLiteral("10b\"10_1001_1111\"".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position(1, 17));

        let contents = "12SX\"F-\";";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::BitStrLiteral(BitStrLiteral("12SX\"F-\"".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position(1, 8));
    }

    #[test]
    fn lex_numeric() {
        let contents = "32)";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("32".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), ")");

        let contents = "32_000;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("32_000".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), ";");

        let contents = "0.456";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("0.456".to_owned())));

        let contents = "6.023E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Decimal("6.023E+24".to_owned())));

        let contents = "2#6.023#E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("2#6.023#E+24".to_owned())));

        let contents = "16#F.FF#E+2";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("16#F.FF#E+2".to_owned())));

        let contents = "2#1.1111_1111_111#E11";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("2#1.1111_1111_111#E11".to_owned())));

        let contents = "016#0FF#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("016#0FF#".to_owned())));

        // '#' can be replaced by ':' if done in both occurences - VHDL 1993 LRM p180
        let contents = "016:0FF:";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).unwrap(), VHDLToken::AbstLiteral(AbstLiteral::Based("016:0FF:".to_owned())));

        let contents = "016:0FF#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_numeric(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_single_comment() {
        let contents = "\
--here is a vhdl comment";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume(); // already determined first dash
        assert_eq!(consume_comment(&mut tc).unwrap(), VHDLToken::Comment(Comment::Single("here is a vhdl comment".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position(1, 24));

        let contents = "\
--here is a vhdl comment
entity fa is end entity;";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume(); // already determined first dash
        assert_eq!(consume_comment(&mut tc).unwrap(), VHDLToken::Comment(Comment::Single("here is a vhdl comment".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), "entity fa is end entity;");
        assert_eq!(tc.locate(), &Position(2, 0));
    }

    #[test]
    fn lex_delim_comment() {
        let contents = "\
/* here is a vhdl 
delimited-line comment. Look at all the space! */;";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume();
        assert_eq!(consume_delim_comment(&mut tc).unwrap(), VHDLToken::Comment(Comment::Delimited(" here is a vhdl 
delimited-line comment. Look at all the space! ".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position(2, 49));

        let contents = "/* here is a vhdl comment";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume();
        assert_eq!(consume_delim_comment(&mut tc).is_err(), true);
    }

    #[test]
    fn lex_char_literal() {
        let contents = "1'";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_char_lit(&mut tc).unwrap(), VHDLToken::CharLiteral(Character("1".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position(1, 2));

        let contents = "12'";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_char_lit(&mut tc).is_err(), true);
    }

    #[test]
    fn lex_expon() {
        let contents = "E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_exponent(&mut tc, c0).unwrap(), "E+24");
        assert_eq!(tc.as_ref().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position(1, 4));

        let contents = "e6;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_exponent(&mut tc, c0).unwrap(), "e6");
        assert_eq!(tc.as_ref().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position(1, 2));

        let contents = "e-12;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_exponent(&mut tc, c0).unwrap(), "e-12");
        assert_eq!(tc.as_ref().clone().collect::<String>(), ";");

        // negative test cases
        let contents = "e-;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_exponent(&mut tc, c0).is_err(), true);

        let contents = "e+2_;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_exponent(&mut tc, c0).is_err(), true);

        let contents = "e";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_exponent(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_integer() {
        // allow bit string literal to be none
        let contents = "";
        // testing using digit prod. rule "graphic"
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_graphic).unwrap(), "");
        assert_eq!(tc.as_ref().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position(1, 0));

        let contents = "234";
        // testing using digit prod. rule "integer"
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_digit).unwrap(), "234");
        assert_eq!(tc.as_ref().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position(1, 3));

        let contents = "1_2_345 ";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_digit).unwrap(), "1_2_345");
        assert_eq!(tc.as_ref().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position(1, 7));

        let contents = "23__4";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_digit).is_err(), true); // double underscore

        let contents = "_24";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_digit).is_err(), true); // leading underscore

        let contents = "_23_4";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, Some('1'), char_set::is_digit).is_ok(), true); 

        // testing using extended_digit prod. rule "based_integer"
        let contents = "abcd_FFFF_0021";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_extended_digit).unwrap(), "abcd_FFFF_0021");

        // testing using graphic_char prod. rule "bit_value"
        let contents = "XXXX_01LH_F--1";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(consume_integer(&mut tc, None, char_set::is_graphic).unwrap(), "XXXX_01LH_F--1");
    }

    #[test]
    fn lex_identifier() {
        let words = "entity is";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_word(&mut tc, c0).unwrap(), VHDLToken::Entity);
        assert_eq!(tc.as_ref().clone().collect::<String>(), " is");
        assert_eq!(tc.locate(), &Position(1, 6));

        let words = "std_logic_1164.all;";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_word(&mut tc, c0).unwrap(), VHDLToken::Identifier(Identifier::Basic("std_logic_1164".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), ".all;");
        assert_eq!(tc.locate(), &Position(1, 14));

        let words = "ready_OUT<=";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_word(&mut tc, c0).unwrap(), VHDLToken::Identifier(Identifier::Basic("ready_OUT".to_owned())));
        assert_eq!(tc.as_ref().clone().collect::<String>(), "<=");
        assert_eq!(tc.locate(), &Position(1, 9));
    }

    #[test]
    fn lex_literal() {
        let contents = "\" go Gators! \" ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), " go Gators! ");
        assert_eq!(tc.as_ref().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position(1, 14));

        let contents = "\" go \"\"to\"\"\" ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), " go \"to\"");
        assert_eq!(tc.as_ref().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position(1, 12));

        let contents = "\"go ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).is_err(), true); // no closing quote
    }

    #[test]
    fn lex_literal_2() {
        let contents = "\"Setup time is too short\"more text";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), "Setup time is too short");
        assert_eq!(tc.as_ref().clone().collect::<String>(), "more text");
        assert_eq!(tc.locate(), &Position(1, 25));

        let contents = "\"\"\"\"\"\"";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), "\"\"");
        assert_eq!(tc.locate(), &Position(1, 6));

        let contents = "\" go \"\"gators\"\" from UF! \"";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), " go \"gators\" from UF! ");
        assert_eq!(tc.locate(), &Position(1, 26));

        let contents = "\\VHDL\\";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), "VHDL");

        let contents = "\\a\\\\b\\more text afterward";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(consume_literal(&mut tc, &c0).unwrap(), "a\\b");
        // verify the stream is left in the correct state
        assert_eq!(tc.as_ref().clone().collect::<String>(), "more text afterward");
    }

    #[test]
    fn ignore_case_cmp() {
        let s0 = "ABC";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), true);

        // negative case: different lengths
        let s0 = "ABCD";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // negative case: different letter order
        let s0 = "cba";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // VHDL-2008 LRM p226
        let s0 = "ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ";
        let s1 = "abcdefghijklmnopqrstuvwxyzàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþ";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // these 2 letters do not have upper-case equivalents
        let s0 = "ß";
        let s1 = "ÿ";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);
    }

    mod vhdl {
        use super::*;

        #[test]
        fn easy_tokens() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
entity fa is end entity;";
            let tokens: Vec<VHDLToken> = VHDLTokenizer::tokenize(s)
                .into_iter()
                .map(|f| { f.take() })
                .collect();
            assert_eq!(tokens, vec![
                Entity,
                Identifier(vhdl::Identifier::Basic("fa".to_owned())),
                Is,
                End,
                Entity,
                Terminator,
                EOF,
            ]);
        }

        #[test]
        fn comment_token() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
-- here is a vhdl single-line comment!";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Comment(vhdl::Comment::Single(" here is a vhdl single-line comment!".to_owned())), Position(1, 1)),
                Token::new(EOF, Position(1, 39)),
            ]);
        }

        #[test]
        fn comment_token_delim() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
/* here is a vhdl 
    delimited-line comment. Look at all the space! */";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Comment(vhdl::Comment::Delimited(" here is a vhdl 
    delimited-line comment. Look at all the space! ".to_owned())), Position(1, 1)),
                Token::new(EOF, Position(2, 54)),
            ]);
        }

        #[test]
        fn char_literal() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
signal magic_num : std_logic := '1';";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Signal, Position(1, 1)),
                Token::new(Identifier(vhdl::Identifier::Basic("magic_num".to_owned())), Position(1, 8)),
                Token::new(Colon, Position(1, 18)),
                Token::new(Identifier(vhdl::Identifier::Basic("std_logic".to_owned())), Position(1, 20)),
                Token::new(VarAssign, Position(1, 30)),
                Token::new(CharLiteral(vhdl::Character("1".to_owned())), Position(1, 33)),
                Token::new(Terminator, Position(1, 36)),
                Token::new(EOF, Position(1, 37)),
            ]);
        }

        #[test]
        fn easy_locations() {
            use crate::core::vhdl::*;
            let s = "\
entity fa is end entity;";
            let tokens: Vec<Position> = VHDLTokenizer::tokenize(s)
                .into_iter()
                .map(|f| { f.locate().clone() })
                .collect();
            assert_eq!(tokens, vec![
                Position(1, 1),  // 1:1 keyword: entity
                Position(1, 8),  // 1:8 basic identifier: fa
                Position(1, 11), // 1:11 keyword: is
                Position(1, 14), // 1:14 keyword: end
                Position(1, 18), // 1:18 keyword: entity
                Position(1, 24), // 1:24 delimiter: ;
                Position(1, 25), // 1:25 eof
            ]);  
        }

        #[test]
        fn read_delimiter_single() {
            use super::VHDLToken::*;

            let contents = "&";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(Ampersand));
            assert_eq!(tc.as_ref().clone().collect::<String>(), "");
            assert_eq!(tc.locate(), &Position(1, 1));

            let contents = "?";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(Question));
            assert_eq!(tc.as_ref().clone().collect::<String>(), "");
            assert_eq!(tc.locate(), &Position(1, 1));

            let contents = "< MAX_COUNT";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(Lt));
            assert_eq!(tc.as_ref().clone().collect::<String>(), " MAX_COUNT");
            assert_eq!(tc.locate(), &Position(1, 1));

            let contents = ");";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(ParenR));
            assert_eq!(tc.as_ref().clone().collect::<String>(), ";");
            assert_eq!(tc.locate(), &Position(1, 1));
        }

        #[test]
        fn read_delimiter_none() {
            let contents = "fa";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None).is_err(), true);
            assert_eq!(tc.as_ref().clone().collect::<String>(), "a");
            assert_eq!(tc.locate(), &Position(1, 1));
        }

        #[test]
        fn read_delimiter_double() {
            use super::VHDLToken::*;

            let contents = "<=";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(SigAssign));
            assert_eq!(tc.as_ref().clone().collect::<String>(), "");
            assert_eq!(tc.locate(), &Position(1, 2));

            let contents = "**WIDTH";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(DoubleStar));
            assert_eq!(tc.as_ref().clone().collect::<String>(), "WIDTH");
            assert_eq!(tc.locate(), &Position(1, 2));
        }

        #[test]
        fn read_delimiter_triple() {
            use super::VHDLToken::*;

            let contents = "<=>";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(SigAssoc));
            assert_eq!(tc.as_ref().clone().collect::<String>(), "");
            assert_eq!(tc.locate(), &Position(1, 3));

            let contents = "?/= MAGIC_NUM";
            let mut tc = TrainCar::new(contents.chars());
            assert_eq!(collect_delimiter(&mut tc, None), Ok(MatchNE));
            assert_eq!(tc.as_ref().clone().collect::<String>(), " MAGIC_NUM");
            assert_eq!(tc.locate(), &Position(1, 3));
        }

        #[test]
        fn match_delimiter() {
            use super::VHDLToken::*;

            let word = "<=";
            assert_eq!(VHDLToken::match_delimiter(word), Ok(SigAssign));

            let word = "-";
            assert_eq!(VHDLToken::match_delimiter(word), Ok(Dash));

            let word = "<=>";
            assert_eq!(VHDLToken::match_delimiter(word), Ok(SigAssoc));

            let word = "^";
            assert_eq!(VHDLToken::match_delimiter(word).is_err(), true);

            let word = "entity";
            assert_eq!(VHDLToken::match_delimiter(word).is_err(), true);
        }

        #[test]
        fn match_reserved_idenifier() {
            use super::VHDLToken::*;

            let word = "END";
            assert_eq!(VHDLToken::match_keyword(word), Some(End));

            let word = "EnTITY";
            assert_eq!(VHDLToken::match_keyword(word), Some(Entity));

            let word = "entitys";
            assert_eq!(VHDLToken::match_keyword(word), None);

            let word = "<=";
            assert_eq!(VHDLToken::match_keyword(word), None);
        }

        #[test]
        fn is_sep() {
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
        fn eq_identifiers() {
            let id0 = Identifier::Basic("fa".to_owned());
            let id1 = Identifier::Basic("Fa".to_owned());
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
        }

        #[test]
        #[ignore]
        fn nor_gate_design_code() {
            let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity nor_gate is 
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
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx\"\"
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 0; --8x\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl;";
            let vhdl = VHDLTokenizer::tokenize(&s);
            let vhdl = VHDLTokenizer { inner: vhdl };
            println!("{:?}", vhdl);
            todo!()
        }
    }

    mod position {
        use super::*;

        #[test]
        fn moving_position() {
            let mut pos = Position::new();
            assert_eq!(pos, Position(1, 0));
            pos.next_col();
            assert_eq!(pos, Position(1, 1));
            pos.next_col();
            assert_eq!(pos, Position(1, 2));
            pos.next_line();
            assert_eq!(pos, Position(2, 0));
            pos.next_line();
            assert_eq!(pos, Position(3, 0));
        }
    }
}