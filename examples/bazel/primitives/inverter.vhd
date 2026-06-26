library ieee;
use ieee.std_logic_1164.all;

entity inverter is
  port (
    a: in  std_logic;
    y: out std_logic
  );
end entity;

architecture rtl of inverter is
begin
  y <= not a;
end architecture;
