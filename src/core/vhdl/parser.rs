use std::fmt::Display;
use crate::core::lexer::Token;

pub trait Parse<T> {
    type SymbolType;
    type Err;

    fn parse(tokens: Vec<Token<T>>) -> Vec<Result<Symbol<Self::SymbolType>, SymbolError<Self::Err>>> 
        where <Self as Parse<T>>::Err: Display;
}

#[derive(Debug, PartialEq)]
pub struct Symbol<T> {
    stype: T,
}

#[derive(Debug, PartialEq)]
pub struct SymbolError<T: Display> {
    err: T,
}

impl<T: Display> SymbolError<T> {
    /// Creates a new `TokenError` struct at position `loc` with error `T`.
    pub fn new(err: T) -> Self {
        Self { err: err }
    }
}

impl<T: Display> Display for SymbolError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

#[derive(Debug, PartialEq)]
enum VHDLSymbol {
    Entity,
    Architecture
}

#[derive(Debug, PartialEq)]
struct VHDLParser {
    symbols: Vec<Symbol<VHDLSymbol>>,
}

use crate::core::vhdl::vhdl::*;

impl Parse<VHDLToken> for VHDLParser {
    type SymbolType = VHDLSymbol;
    type Err = String;

    fn parse(tokens: Vec<Token<VHDLToken>>) -> Vec<Result<Symbol<Self::SymbolType>, SymbolError<Self::Err>>>
        where <Self as Parse<VHDLToken>>::Err: Display {
        
        let mut tokens = tokens.into_iter().peekable();

        while let Some(t) = tokens.next() {
            // create entity symbol
            if t.as_ref().check_keyword(&Keyword::Entity) {
                let name = VHDLSymbol::parse_entity(&mut tokens);
                println!("!!!! INFO: detected primary design unit entity \"{}\"", name);
            // create architecture symbol
            } else if t.as_ref().check_keyword(&Keyword::Architecture) {
                let (arch, entity) = VHDLSymbol::parse_architecture(&mut tokens);
                println!("!!!! INFO: detected architecture \"{}\" for entity \"{}\"", arch, entity);
            // create configuration symbol
            } else if t.as_ref().check_keyword(&Keyword::Configuration) {
                let (config, entity) = VHDLSymbol::parse_architecture(&mut tokens);
                println!("!!!! INFO: detected configuration \"{}\" for entity \"{}\"", config, entity);
            // create package symbol
            } else if t.as_ref().check_keyword(&Keyword::Package) {
                let pack = if &VHDLToken::Keyword(Keyword::Body) == tokens.peek().unwrap().as_type() {
                    // take the 'body' keyword
                    tokens.next();
                    VHDLSymbol::parse_entity(&mut tokens)
                } else {
                    VHDLSymbol::parse_entity(&mut tokens)
                };
                println!("!!!! INFO: detected package \"{}\"", pack);
            // otherwise take a statement (probably by mistake/errors in user's vhdl code)
            } else {
                let mut stmt: Statement = Statement::new();
                stmt.0.push(t);
                stmt.0.append(&mut VHDLSymbol::compose_statement(&mut tokens).0);
                println!("global statement: {:?}", stmt);
            }

        }
        // detect entity keyword
        todo!()
    }
}

use std::iter::Peekable;

/// A `Statement` is a vector of tokens, similiar to how a `String` is a vector
/// of characters.
#[derive(PartialEq)]
struct Statement(Vec<Token<VHDLToken>>);

impl std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.0 {
            write!(f, "{} ", t.as_ref().to_string())?
        }
        Ok(())
    }
}

struct Node {
    sym: VHDLSymbol,
    childern: Box<Vec<Node>>,
}

impl Statement {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn as_types(&self) -> Vec<&VHDLToken> {
        self.0.iter().map(|f| f.as_type() ).collect()
    }
}

impl VHDLSymbol {
    fn parse_entity<I>(tokens: &mut Peekable<I>) -> Identifier 
    where I: Iterator<Item=Token<VHDLToken>>  {
        // take entity name
        let entity_name = tokens.next().take().unwrap().take();
        println!("*--- unit {}", entity_name);
        VHDLSymbol::parse_declaration(tokens);
        match entity_name {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        }
    }

    /// Detects identifiers instantiated in the architecture statement sections.
    /// Assumes the next token to consume is the COLON delimiter.
    fn parse_instantiation(statement: Statement) -> Option<Identifier> {
        let mut tokens = statement.0.into_iter();
        // force identifier
        tokens.next()?.take().get_identifier()?;
        // force colon
        if tokens.next()?.take().check_delimiter(&Delimiter::Colon) == false { return None };
        // check what is instantiated
        match tokens.next()?.take() {
            VHDLToken::Identifier(id) => Some(id),
            VHDLToken::Keyword(kw) => {
                if kw == Keyword::Component || kw == Keyword::Entity || kw == Keyword::Configuration {
                    match tokens.next()?.take() {
                        VHDLToken::Identifier(id) => Some(id),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_architecture<I>(tokens: &mut Peekable<I>) -> (Identifier, Identifier) 
        where I: Iterator<Item=Token<VHDLToken>>  {
        let arch_name = match tokens.next().take().unwrap().take() {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        };
        let entity_name = VHDLSymbol::parse_primary_design_unit(tokens);
        println!("*--- unit {}", arch_name);
        VHDLSymbol::parse_declaration(tokens);
        (arch_name, entity_name)
    }

    /// Parses together a series of tokens into a single `Statement`.
    /// 
    /// Statements end on a ';' and do not include the ';' token. If the EOF
    /// is reached before completing a statement, it is omitted and a blank
    /// statement is returned.
    fn compose_statement<I>(tokens: &mut Peekable<I>) -> Statement 
        where I: Iterator<Item=Token<VHDLToken>>  {
        let mut statement = Statement::new();
        while let Some(t) = tokens.next() {
            // exit upon encountering terminator ';'
            if t.as_type().check_delimiter(&Delimiter::Terminator) {
                return statement
            // extra keywords to help break up statements early
            } else if t.as_type().check_keyword(&Keyword::Generate) || 
                t.as_type().check_keyword(&Keyword::Begin) {
                statement.0.push(t);
                return statement
            } else if statement.0.first().is_some() && statement.0.first().unwrap().as_type().check_keyword(&Keyword::When) &&
                t.as_type().check_delimiter(&Delimiter::Arrow) {
                statement.0.push(t);
                return statement
            } else {
                statement.0.push(t);
            }
        }
        Statement::new()
    }

    fn parse_primary_design_unit<I>(tokens: &mut Peekable<I>) -> Identifier
    where I: Iterator<Item=Token<VHDLToken>>  {
        // force taking the 'of' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Of) == false {
            panic!("expecting 'of' keyword")
        }
        // return the name of the primary design unit
        match tokens.next().take().unwrap().take() {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        }
    }

    fn parse_declaration<I>(tokens: &mut Peekable<I>) -> Vec<Statement>
        where I: Iterator<Item=Token<VHDLToken>>  {
        println!("*--- declaration section");
        // force taking the 'is' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            panic!("expecting 'is' keyword")
        }
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::Begin) {
                Self::parse_body(tokens);
                break;
            } else if t.as_type().check_keyword(&Keyword::End) {
                let stmt = Self::compose_statement(tokens);
                println!("{:?}", stmt);
                if Self::is_closer(stmt) { break; }
            // compose a statement @TODO handle recursion for detecting subprogram calls such as 'function' keyword
            } else {
                let stmt = Self::compose_statement(tokens);
                println!("{:?}", stmt);
            }
        }
        Vec::new()
    }

    fn is_closer(stmt: Statement) -> bool {
        let keyword = if let Some(t) = stmt.0.get(1) {
            t.as_type()
        } else {
            return false // @TODO make sure "end;" will end up being valid
        };
        match keyword {
            // list mandatory keywords expected after the 'end' keyword
            VHDLToken::Keyword(kw) => match kw {
                Keyword::Loop | Keyword::Generate | Keyword::Process |
                Keyword::Postponed | Keyword::If | Keyword::Block | 
                Keyword::Protected | Keyword::Record | Keyword::Case | 
                Keyword::Component | Keyword::For => false,
                _ => true,
            },
            _ => true,
        }
    }

    /// Parses a body, expecting the first token to be the BEGIN keyword.
    /// 
    /// Builds statements until finds the END keyword statement
    fn parse_body<I>(tokens: &mut Peekable<I>) -> Vec<Statement>
        where I: Iterator<Item=Token<VHDLToken>>  {
        println!("*--- statement section");
        // force taking the 'begin' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Begin) == false{
            panic!("expecting 'begin' keyword")
        }
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) {
                let stmt = Self::compose_statement(tokens);
                println!("{:?}", stmt);
                if Self::is_closer(stmt) { break; }
            } else {
                let stmt = Self::compose_statement(tokens);
                println!("{:?}", stmt);
                // check if statement is an instantiation
                if let Some(inst) = Self::parse_instantiation(stmt) {
                    println!("**** INFO: Detected dependency \"{}\"", inst);
                }
            }
        }
        Vec::new()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compose_statement() {
        let s = "a : in std_logic_vector(3 downto 0);";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).as_types(), vec![
            &VHDLToken::Identifier(Identifier::Basic("a".to_owned())),
            &VHDLToken::Delimiter(Delimiter::Colon),
            &VHDLToken::Keyword(Keyword::In),
            &VHDLToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenL),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("3".to_owned())),
            &VHDLToken::Keyword(Keyword::Downto),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("0".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenR),
            // @TODO include terminator in statement?
        ]);

        let s = "a : in std_logic_vector(3 downto 0); ready: out std_logic);";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).as_types(), vec![
            &VHDLToken::Identifier(Identifier::Basic("a".to_owned())),
            &VHDLToken::Delimiter(Delimiter::Colon),
            &VHDLToken::Keyword(Keyword::In),
            &VHDLToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenL),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("3".to_owned())),
            &VHDLToken::Keyword(Keyword::Downto),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("0".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenR),
        ]);
    }

    #[test]
    #[ignore]
    fn parse_basic() {
        let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity nor_gate is -- comment on this line
    generic(
        N: positive
    );
    port(
        a : in std_logic_vector(N-1 downto 0);
        b : in std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
begin
end entity nor_gate;

architecture rtl of nor_gate is
    constant GO_ADDR_MMAP:integer:=2#001_1100.001#E14;
    constant freq_hz : unsigned := 50_000_000;
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx\"\";
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 0; -- 8c\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl; /* long comment */

entity nor_gate_tb is end;

architecture tb of nor_gate_tb is 
-- declare DUT component
component nor_gate 
	generic(
		N: positive
	);
	port(
		a: in  std_logic_vector(N-1 downto 0);
		b: in  std_logic_vector(N-1 downto 0);
		c: out std_logic_vector(N-1 downto 0)
	);
end component nor_gate;
begin 
	DUT : 
    
    
    nor_gate 
    generic map     (
		N   => N
	) port map(
		a => w_a,
		b => w_b,
		c => w_c
	);

end;

package P is

function F return INTEGER;
attribute FOREIGN of F: function is \"implementation-dependent information\"; 

end package P;

package TimeConstants is 
constant tPLH : Time := 9 ns;
constant tPHL : Time := 10 ns;
constant tPLZ :  Time := 12 ns;
constant tPZL :  Time := 7 ns;
constant tPHZ :  Time := 8 ns;
constant tPZH : Time := 8 ns;
end TimeConstants ;

package body TriState is
    function BitVal (Value: Tri) return Bit is
        constant Bits : Bit_Vector := \"0100\"; 
    begin
        return Bits(Tri'Pos(Value)); 
    end;

    function TriVal (Value: Bit) return Tri is 
    begin
        return Tri'Val(Bit'Pos(Value)); 
    end;

    function Resolve (Sources: TriVector) return Tri is 
        variable V: Tri := 'Z';
    begin
        for i in Sources'Range loop
            if Sources(i) /= 'Z' then 
                if V = 'Z' then
                    V := Sources(i);
                else 
                    return 'E';
                end if; 
            end if;
        end loop; 
        return V;
    end;

end package body TriState;

architecture test of nor_gate is 

begin 

GEN_ADD: for I in 0 to 7 generate

LOWER_BIT: if I=0 generate
  U0: HALFADD port map
     (A(I),B(I),S(I),C(I));
end generate LOWER_BIT;

UPPER_BITS: if I>0 generate
  UX: FULLADD port map
     (A(I),B(I),C(I-1),S(I),C(I));
end generate UPPER_BITS;

end generate GEN_ADD;

end architecture;

architecture rtl of complex_multiplier is
begin
mult_structure : case implementation generate when single_cycle =>
signal real_pp1, real_pp2 : ...;
...
begin
real_mult1 : component multiplier
port map ( ... ); ...
end;
when multicycle =>
signal real_pp1, real_pp2 : ...;
...
begin
mult : component multiplier
port map ( ... );
end;
when pipelined => mult1 : component multiplier
port map ( ... );
end;
end generate mutl_structure;
end architecture rtl;
";
        let _ = VHDLParser::parse(VHDLTokenizer::from_source_code(&s).into_tokens());
        panic!("manually inspect token list")
    }
}