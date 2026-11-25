library ieee;
use ieee.std_logic_1164.all;
library techmap;
use techmap.allmul.all;

entity techmult is
  port (clk : in std_logic;
        a, b : in std_logic_vector(15 downto 0);
        p : out std_logic_vector(31 downto 0));
end;

architecture rtl of techmult is
begin
  core : gen_mult
    generic map (width => 16)
    port map (a => a, b => b, p => p);
end;
