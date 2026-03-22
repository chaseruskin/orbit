library ieee;
use ieee.std_logic_1164.all;

entity multi_str_case is
  generic(
    SOME_GENERIC : string := "CASE1";
    SOME_OTHER_GENERIC : string := "CASE1"
  );
  port(
    dat1 : out std_logic;
    dat2 : out std_logic
  );
end entity;

architecture rtl of multi_str_case is
begin

    testgen1 : case SOME_GENERIC generate
      when "CASE1" =>
        -- some RTL instantiated here
        u_foo1 : entity work.foo1
          port map (
            dat => dat1
          );
      when others =>
        -- some default statement here
        dat1 <= '0';
    end generate testgen1;

    testgen2 : case SOME_OTHER_GENERIC generate
      when "CASE1" =>
        -- some other RTL instantiated here
        u_foo2 : entity work.foo2
          port map (
            dat => dat2
          );
      when others =>
        -- some default statement here
        dat2 <= '0';
    end generate testgen2;

end architecture;