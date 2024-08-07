library ieee;
use ieee.std_logic_1164.all;

package casting is

    --! Returns a string representation of logic vector to output to console.
    function to_str(slv: std_logic_vector) return string;

    --! Returns a string representation of logic bit to output to console.
    function to_str(sl: std_logic) return string;

    --! Casts a character `c` to a logical '1' or '0'. Anything not the character
    --! '1' maps to a logical '0'.
    function char_to_logic(c: character) return std_logic;

end package;


package body casting is

    function to_str(slv: std_logic_vector) return string is
        variable str : string(1 to slv'length);
        variable str_index : positive := 1;
        variable sl_bit : std_logic;
    begin
        for ii in slv'range loop
            sl_bit := slv(ii);
            if sl_bit = '1' then
                str(str_index) := '1';
            elsif sl_bit = '0' then
                str(str_index) := '0';
            elsif sl_bit = 'X' then
                str(str_index) := 'X';
            elsif sl_bit = 'U' then
                str(str_index) := 'U';
            else
                str(str_index) := '?';
            end if;
            str_index := str_index + 1;
        end loop;
        return str;
        -- return integer'image(slv'length) & "b'" & str;
    end function;


    function to_str(sl: std_logic) return string is
        variable str : string(1 to 1);
    begin
        if sl = '1' then
            str(1) := '1';
        elsif sl = '0' then
            str(1) := '0';
        elsif sl = 'X' then
            str(1) := 'X';
        elsif sl = 'U' then
            str(1) := 'U';
        else
            str(1) := '?';
        end if;
        return str;
    end function;


    function char_to_logic(c: character) return std_logic is
    begin
        if(c = '1') then
            return '1';
        else
            return '0';
        end if;
    end function;

end package body;