
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

  component b
    port(
      din_0: in std_logic;
      din_1: in std_logic;
      sel: in std_logic;
      mux_out: out std_logic
    );
  end component;

begin

  u_mux: b
    port map(
      din_0 => x,
      din_1 => y,
      sel => '0',
      mux_out => q
    );

end architecture;