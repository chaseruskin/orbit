library ieee;
use ieee.std_logic_1164.all;

entity if_gen is 
    port (
        vec: out std_logic_vector(3 downto 0)
    );
end entity;


architecture rtl of if_gen is

    constant cond: bool := true;
begin
    sub_entity: entity work.sub1
    port (
        vec => vec
    );
    
    foo: if cond = true generate
    begin
        sub_entity: entity work.sub2
            port (
                vec => vec
            );
    end generate;

end architecture;