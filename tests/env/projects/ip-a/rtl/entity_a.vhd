--! ****************************************************************************
--! Entity:     entity_a
--! Engineer:   Chase Ruskin
--! Details: 
--!     Outputs a high logic level ('1').
--! ****************************************************************************
library ieee;
use ieee.std_logic_1164.all;

entity entity_a is
port (
    high : out std_logic
);
end entity entity_a;

architecture rtl of entity_a is

    component dupe
        port (
            data   : in std_logic;
            data_a : out std_logic;
            data_b : out std_logic
        );
    end component;

    signal data   : std_logic;
    signal data_a : std_logic;
    signal data_b : std_logic;

begin   

    data <= '1';

    uX : dupe port map (
        data   => data,
        data_a => data_a,
        data_b => data_b
    );

    high <= data_a and data_b;
end architecture;