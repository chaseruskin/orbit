entity abc is
  generic (
    WIDTH   : integer := 16#120#;
    BITS    : integer := 20;
    ROUNDER : real := 1.23;
    WORD    : string := "hello world!"
  );
  port (
    a : in std_logic;
    b : in std_logic := 'L';
    f : in std_logic_vector(2 downto 0) := "101";
    c : out std_logic_vector(WIDTH-1 downto 0) := (others => '0');
    d : out std_logic_vector(3 downto 0) := X"F";
    e : in std_logic := 'Z'
  );
end entity;

architecture rtl of abc is
begin


end architecture;