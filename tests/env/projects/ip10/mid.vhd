library ieee;
use ieee.std_logic_1164.all;

entity mid is
port (
  data: out std_logic
);
end entity;

architecture rtl of mid is
begin

  u0: entity work.d10
    port map(
      data => data
    );

end architecture;