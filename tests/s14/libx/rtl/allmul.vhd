library ieee;
use ieee.std_logic_1164.all;

package allmul is
  component gen_mult
    generic (width : positive := 16);
    port (a, b : in std_logic_vector(width-1 downto 0);
          p : out std_logic_vector(2*width-1 downto 0));
  end component;
end package;
