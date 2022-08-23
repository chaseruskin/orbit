library ieee;
use ieee.std_logic_1164.all;

entity and_gate is
    port(
        a : in  std_logic;
        b : in  std_logic;
        q : out std_logic
    );
end entity;

architecture rtl of and_gate is 
begin 
    q <= a and b;
    
end architecture;