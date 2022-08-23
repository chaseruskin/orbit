--------------------------------------------------------------------------------
--! Project: ks-tech.rary.gates
--! Author: Chase Ruskin
--! Entity: or_gate
--! About:
--!     Performs logical 'or' operation with variable width.
--------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;

entity or_gate is
    generic (
        N : positive := 8
    );
    port (
        a : in  std_logic_vector(N-1 downto 0);
        b : in  std_logic_vector(N-1 downto 0);
        q : out std_logic_vector(N-1 downto 0)
    );
end entity;

architecture rtl of or_gate is
begin

    q <= a or b;

end architecture;


architecture other of or_gate is
begin

    q <= b or a;

end architecture;

