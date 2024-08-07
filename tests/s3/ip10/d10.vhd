library ieee;
use ieee.std_logic_1164.all;

entity d10 is
port (
  data: out std_logic
);
end entity;

architecture rtl of d10 is
begin

  data <= '1';

end architecture;