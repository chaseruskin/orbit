library ieee;
use ieee.std_logic_1164.all;
library work;

entity circuit_tb is 
end entity circuit_tb;


architecture sim of circuit_tb is
    signal clk   : std_logic;
    signal d_in  : std_logic_vector(3 downto 0);
    signal d_out : std_logic_vector(3 downto 0);
begin
    
    DUT : entity work.circuit
    port map (
        clk   => clk,
        d_in  => d_in,
        d_out => d_out
    );

end architecture sim;