
library ieee;
use ieee.std_logic_1164.all;

entity a is
  port (
    x: in std_logic;
    y: in std_logic;
    q: out std_logic
  );
end entity;

architecture gp of a is

begin

  q <= x and y;

end architecture;