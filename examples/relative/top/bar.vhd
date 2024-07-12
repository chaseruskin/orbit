-- Project: top
-- Entity: bar

library ieee;
use ieee.std_logic_1164.all;
library sub;

entity bar is 
  port(
    rdy: out std_logic;
  );
end entity;

architecture gp of bar is
begin

  u0: entity sub.foo
  port map(
    rdy => rdy
  );

end architecture;
