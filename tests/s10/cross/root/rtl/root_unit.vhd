library ieee;
use ieee.std_logic_1164.all;
library work;
use work.config.all;
library dep4;

entity root_unit is
port (
    clk : in std_logic
);
end entity root_unit;

architecture rtl of root_unit is
    constant LOCAL_WIDTH : integer := WIDTH;
begin
    u_dep : entity dep4.dep_unit port map ( clk => clk );
end architecture;
