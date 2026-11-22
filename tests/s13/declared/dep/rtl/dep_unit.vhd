library ieee;
use ieee.std_logic_1164.all;
library work;
use work.config.all;

entity dep_unit is
port (
    clk : in std_logic
);
end entity dep_unit;

architecture rtl of dep_unit is
    constant DATA_WIDTH : integer := WIDTH;
begin
end architecture;
