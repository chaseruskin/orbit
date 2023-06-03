--! ****************************************************************************
--! Entity:     entity_c
--! Engineer:   Chase Ruskin
--! Details: 
--!     Outputs a low logic level ('0').
--! ****************************************************************************
library ieee;
use ieee.std_logic_1164.all;
library work;

entity entity_c is
port (
    high : out std_logic;
    low : out std_logic
);
end entity entity_c;

architecture rtl of entity_c is

    component entity_a
        port (
            high : out std_logic
        );
    end component;

    signal high_i : std_logic;

    signal data   : std_logic;
    signal data_a : std_logic;
    signal data_b : std_logic;

begin
    high <= high_i;
    data <= '0';

    u_ea : entity_a
    port map (
        high => high_i
    );

    u_dupe : entity work.dupe
    port map (
        data   => data,
        data_a => data_a,
        data_b => data_b
    );

    low <= data_a or data_b;
end architecture;