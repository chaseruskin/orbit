--! ****************************************************************************
--! Entity:     dupe
--! Engineer:   Chase Ruskin
--! Details: 
--!     Splits the incoming signal onto two different wires.
--! ****************************************************************************
library ieee;
use ieee.std_logic_1164.all;

entity dupe is
port (
    data : in std_logic;
    data_a : out std_logic;
    data_b : out std_logic
);
end entity dupe;

architecture rtl of dupe is
begin
    data_a <= data;
    data_b <= data;
end architecture;