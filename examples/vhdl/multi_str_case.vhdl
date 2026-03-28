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

  function ADD(A, B, CIN : BIT) return BIT_VECTOR is
    variable S, COUT : BIT;
    variable RESULT : BIT_VECTOR(1 downto 0);
  begin
    S := A xor B xor CIN;
    COUT := (A and B) or (A and CIN) or (B and CIN);
    RESULT := COUT & S;
    return RESULT;
  end ADD;

begin

    testgen1 : case SOME_GENERIC generate
      when "CASE1" =>
          b_foo : block
          begin
            -- some RTL instantiated here
            u_foo1 : entity work.foo1
              port map (
                dat => dat1
              );
          end block;
      when others =>
        signal abc : std_logic;
      begin
        b_foo : block
        begin
          -- some default statement here
          dat1 <= '0';
        end block;
      end;
    end generate testgen1;

    testgen2 : case SOME_OTHER_GENERIC generate
      when "CASE1" =>
        signal abc : std_logic;
      begin
        -- some default statement here
        dat2 <= '0';
      end;
      when others =>
      b_foo : block begin
        -- some other RTL instantiated here
        u_foo2 : entity work.foo2
          port map (
            dat => dat2
          );
      end block;
    end generate testgen2;

end architecture;