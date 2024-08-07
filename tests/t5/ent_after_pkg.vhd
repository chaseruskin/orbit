-- entity
library ieee;
use ieee.std_logic_1164.all;

entity ent is 
    port (
        c: out std_logic
    );
end entity;

architecture rtl of ent is
begin
    c <= '1';
end architecture;

-- package
library ieee;
use ieee.std_logic_1164.all;

package simple_pkg is
    constant FOO: integer := 3;
end package;

package body simple_pkg is
    constant FOO: integer := 3;
end package body;

-- testbench
library ieee;
use ieee.std_logic_1164.all;

library work;

entity simple_ent is 
end entity;

architecture rtl of simple_ent is
    signal c : std_logic;
begin
    uut: entity work.ent
        port map (
            c => c
        );
end architecture;