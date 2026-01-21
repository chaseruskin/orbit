entity abc is
port(
  foo: in bit;
  bar: out bit;
);
end entity;

architecture rtl of abc is

  `if G_MY_GENERIC then
    attribute my_attribute_typ : string;
    attribute an_attribute of my_attribute_typ: signal is "value";
  `else
    -- nothing is created
  `end if

  type MyRecord is record
  end record;

begin

  ux : entity work.fa
  port map (
    input1    => input1,
    input2    => input2,
    carry_in  => carry_in,
    sum       => sum,
    carry_out => carry_out
  );

end architecture;