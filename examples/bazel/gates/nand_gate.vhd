library ieee;
use ieee.std_logic_1164.all;

library primitives;
use primitives.and_gate;
use primitives.inverter;

entity nand_gate is
  port (
    a: in  std_logic;
    b: in  std_logic;
    y: out std_logic
  );
end entity;

architecture rtl of nand_gate is
  signal and_out: std_logic;
begin
  u_and: entity primitives.and_gate
    port map (a => a, b => b, y => and_out);

  u_inv: entity primitives.inverter
    port map (a => and_out, y => y);
end architecture;
