library ieee;
use ieee.std_logic_1164.all;
use techmap.aux_pkg.all;

entity gen_mul is
  port (a, b : in std_logic_vector(15 downto 0);
        p : out std_logic_vector(31 downto 0));
end;

architecture rtl of gen_mul is
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
  p <= prod;
end;

library ieee;
use ieee.std_logic_1164.all;

entity gen_mult is
  generic (width : positive := 16);
  port (a, b : in std_logic_vector(width-1 downto 0);
        p : out std_logic_vector(2*width-1 downto 0));
end;

architecture rtl of gen_mult is
begin
  p <= a & b;
end;
