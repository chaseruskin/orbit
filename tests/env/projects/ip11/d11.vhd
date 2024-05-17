library ieee;
use ieee.std_logic_1164.all;

library work;

entity d11 is
port (
  data: out std_logic
);
end entity;

architecture rtl of d11 is
begin

  data <= '1';

  uX: entity work.d10
    port map(
      data => data
    );

end architecture;