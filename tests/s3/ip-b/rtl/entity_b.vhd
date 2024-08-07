--! ****************************************************************************
--! Entity:     entity_b
--! Engineer:   Chase Ruskin
--! Details: 
--!     Outputs a low logic level ('0').
--! ****************************************************************************
library ieee;
use ieee.std_logic_1164.all;

entity entity_b is
port (
    low : out std_logic
);
end entity entity_b;

architecture rtl of entity_b is
begin
    low <= '0';
end architecture;