library ieee;
use ieee.std_logic_1164.all;
library design;
use design.all;

entity testbench is
end;

architecture behav of testbench is
  signal clk : std_logic := '0';
  signal a, b : std_logic_vector(15 downto 0) := (others => '0');
  signal p : std_logic_vector(31 downto 0);
  component top
    port (clk : in std_logic;
          a, b : in std_logic_vector(15 downto 0);
          p : out std_logic_vector(31 downto 0));
  end component;
begin
  dut : top port map (clk => clk, a => a, b => b, p => p);
end;
