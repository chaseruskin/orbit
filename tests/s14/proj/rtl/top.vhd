library ieee;
use ieee.std_logic_1164.all;
library techmap;
use techmap.all;

entity top is
  port (clk : in std_logic;
        a, b : in std_logic_vector(15 downto 0);
        p : out std_logic_vector(31 downto 0));
end;

architecture rtl of top is
  component techmult
    port (clk : in std_logic;
          a, b : in std_logic_vector(15 downto 0);
          p : out std_logic_vector(31 downto 0));
  end component;
begin
  u : techmult port map (clk => clk, a => a, b => b, p => p);
end;
