library ieee;
use ieee.std_logic_1164.all;

entity foo2 is
    port (
        dat : out std_logic
    );
end entity;

architecture rtl of foo2 is
begin

    dat <= '1';

end architecture;