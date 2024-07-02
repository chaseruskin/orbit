--------------------------------------------------------------------------------
-- Project: eel4712c.lab2
-- Author: Chase Ruskin
-- Course: Digital Design - EEL4712C
-- Creation Date: September 20, 2021
-- Entity: decoder7seg
-- Description:
--  Take a 4 bit binary input and translate it to the corresponding output bits
--  to drive a seven segment display in common anode configuration (active-low).
--------------------------------------------------------------------------------

library ieee;
use ieee.std_logic_1164.all;

entity decoder7seg is
    port(
        input  : in  std_logic_vector(3 downto 0);
        output : out std_logic_vector(6 downto 0)
    );
end entity;


architecture rtl of decoder7seg is

    signal segments_i : std_logic_vector(6 downto 0) := (others => '0');
begin
                  --decimal representations
    segments_i <= "0111111" when (input = "0000") else
                  "0000110" when (input = "0001") else
                  "1011011" when (input = "0010") else
                  "1001111" when (input = "0011") else
                  "1100110" when (input = "0100") else
                  "1101101" when (input = "0101") else
                  "1111101" when (input = "0110") else
                  "0000111" when (input = "0111") else
                  "1111111" when (input = "1000") else
                  "1100111" when (input = "1001") else
                  --hex representations
                  "1110111" when (input = "1010") else
                  "1111100" when (input = "1011") else
                  "0111001" when (input = "1100") else
                  "1011110" when (input = "1101") else
                  "1111001" when (input = "1110") else
                  "1110001" when (input = "1111") else
                  "0000000";

    --invert internal signals to correctly display for common anode configuration
    output <= not segments_i;

end architecture;