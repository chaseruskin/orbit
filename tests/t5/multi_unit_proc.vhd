library ieee;
use ieee.std_logic_1164.all;

entity mul_a is
  port (a, b : in std_logic_vector(15 downto 0);
        p : out std_logic_vector(31 downto 0));
end;

architecture rtl of mul_a is
  signal prod : std_logic_vector(31 downto 0);
begin
  comb : process(a, b)
  begin
    if a = b then
      prod <= (others => '0');
    else
      prod <= a & b;
    end if;
  end process;
  reg : process(clk)
  begin
    if rising_edge(clk) then
      p <= prod;
    end if;
  end process;
  p <= prod;
end;

library ieee;
use ieee.std_logic_1164.all;

entity mul_b is
  port (clk : in std_logic;
        p : in std_logic_vector(31 downto 0));
end;

architecture rtl of mul_b is
begin
  reg : process(clk)
  begin
    if rising_edge(clk) then
      null;
    end if;
  end process;
end;
