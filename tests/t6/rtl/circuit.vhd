library ieee;
use ieee.std_logic_1164.all;

entity circuit is 
port (
    clk : in std_logic;
    d_in : in std_logic_vector(3 downto 0);
    d_out : out std_logic_vector(3 downto 0)
);
end entity;


architecture rtl of circuit is 
    signal d_r : std_logic_vector(3 downto 0);
begin

    reg : process(clk) 
    begin
        d_r <= d_in;
    end process;

    d_out <= d_r;

end architecture rtl;