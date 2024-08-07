library ieee;
use ieee.std_logic_1164.all;

entity proced_in_proc is 
    port (
        vec: out std_logic_vector(3 downto 0)
    );
end entity;


architecture rtl of proced_in_proc is

    constant cond: bool := true;

    procedure print_char(c: character) is
    begin
        report c;
    end procedure;
begin

    s0: entity work.sub0
    port (
        vec => vec
    );

    foo0: process

        -- constant bar: integer := 3;

        procedure print_num(i: integer) is
        begin
            report integer'image(i);
        end procedure;
    begin
        print_num(5);
    end process;

    s1: entity work.sub1
    port (
        vec => vec
    );


    foo1: process(sig0, sig1)

        -- constant bar: integer := 3;

        procedure print_num(i: integer) is
        begin
            report integer'image(i);
        end procedure;
    begin
        print_num(5);
    end process;

    s2: entity work.sub2
    port (
        vec => vec
    );

end architecture;