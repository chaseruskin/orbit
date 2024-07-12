-- Project: sub
-- Entity: foo

library ieee;
use ieee.std_logic_1164.all;

entity foo is 
  port(
    rdy: out std_logic
  );
end entity;

architecture gp of foo is
begin
  rdy <= '1';

end architecture;
