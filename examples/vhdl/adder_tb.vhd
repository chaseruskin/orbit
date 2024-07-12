--------------------------------------------------------------------------------
-- Project: eel4712c.lab1
-- Author: Greg Stitt, University of Florida
-- Course: Digital Design - EEL4712C
-- Creation Date: -
-- Entity: adder_tb
-- Description:
--    Verifies 6-bit ripple carry adder design using assert statements and
--  built-in ideal model functions for sum and carry.
--------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity adder_tb is
end adder_tb;

architecture TB of adder_tb is

  signal input1, input2, sum : std_logic_vector(5 downto 0);
  signal carry_in, carry_out : std_logic;

begin  -- TB

  UUT : entity work.adder
    port map (
      input1    => input1,
      input2    => input2,
      carry_in  => carry_in,
      sum       => sum,
      carry_out => carry_out);

  process

    function sum_test (
      constant in1        : integer;
      constant in2        : integer;
      constant f_carry_in : integer)
      return std_logic_vector is
    begin
      return std_logic_vector(to_unsigned((in1+in2+f_carry_in) mod 64, 6));
    end sum_test;

    function carry_test (
      constant in1        : integer;
      constant in2        : integer;
      constant f_carry_in : integer)
      return std_logic is
    begin
      if(in1+in2+f_carry_in > 63) then
        return '1';
      else
        return '0';
      end if;
    end carry_test;

  begin
    -- test all input combinations
    for i in 0 to 63 loop
      for j in 0 to 63 loop
        for k in 0 to 1 loop

          input1   <= std_logic_vector(to_unsigned(i, 6));
          input2   <= std_logic_vector(to_unsigned(j, 6));
          carry_in <= std_logic(to_unsigned(k, 1)(0));
          wait for 40 ns;
          assert(sum = sum_test(i,j,k)) report "Sum incorrect";
          assert(carry_out = carry_test(i,j,k)) report "Carry incorrect";

        end loop;  -- k
      end loop;  -- j
    end loop;  -- i

    report "SIMULATION FINISHED!";
    
    wait;

  end process;

end TB;