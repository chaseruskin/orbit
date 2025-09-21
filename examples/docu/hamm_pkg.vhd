--------------------------------------------------------------------------------
--! Project  : Hamming
--! Engineer : Chase Ruskin
--! Created  : 2022-10-10
--! Entity   : hamm_pkg
--! Details  :
--!     Hamming code package consisting of helper functions and constants.
--------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

--- Hamming code package consisting of helper functions.
package hamm_pkg is

    subtype logic is std_ulogic;
    subtype logics is std_ulogic_vector;

    --- Determines if the `num` is a power of 2. 
    ---
    --- Includes values of 0 and 1.
    function is_pow_2(num: natural) return boolean;

    --- Computes the number of data bits for a hamming-code block.
    function data_size(parity_bits: positive range 2 to positive'high) return positive;

    --- Computes the number of bits in the entire hamming-code block.
    function block_size(parity_bits: positive range 2 to positive'high) return positive;

end package hamm_pkg;


package body hamm_pkg is 

    function is_pow_2(num: natural) return boolean is
        variable temp: natural;
    begin
        temp := num;
        while temp > 2 loop 
            if temp rem 2 /= 0 and temp > 2 then
                return false;
            end if;
            temp := temp / 2;
        end loop;
        return true;
    end function;

    function data_size(parity_bits: positive range 2 to positive'high) return positive is
    begin
        return (2**parity_bits)-parity_bits-1;
    end function;

    function block_size(parity_bits: positive range 2 to positive'high) return positive is
    begin
        return 2**parity_bits;
    end function;

end package body;