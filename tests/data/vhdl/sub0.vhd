library ieee;
use ieee.std_logic_1164.all;

entity sub0 is 
    port (
        vec: out std_logic_vector(3 downto 0)
    );
end entity;

architecture rtl of sub0 is
    signal data : std_logic;
begin

    ss0 : entity work.subsub0
        port map (
            data => data
        );

    vec <= (others => '0');
end architecture;