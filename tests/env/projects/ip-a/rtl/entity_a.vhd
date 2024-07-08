--! ****************************************************************************
--! Entity:     entity_a
--! Engineer:   Chase Ruskin
--! Details: 
--!     Outputs a high logic level ('1').
--! ****************************************************************************
library ieee;
use ieee.std_logic_1164.all;
library dupe;

entity entity_a is
port (
    high : out std_logic
);
end entity entity_a;

architecture rtl of entity_a is

    component dupe2
        port (
            en  : in std_logic;
            led : out std_logic
        );
    end component;

    signal data   : std_logic;
    signal data_a : std_logic;
    signal data_b : std_logic;

begin   

    data <= '1';

    u_vhdl_dupe : entity dupe.dupe(rtl) port map (
        data   => data,
        data_a => data_a,
        data_b => data_b
    );

    u_verilog_dupe : dupe2 port map (
        en => '1',
        led => open
    );

    high <= data_a and data_b;
end architecture;