entity top is
  port (
    ok : out bit
  );
end entity;

architecture sys1 of top is
begin
  ok <= '1';
end architecture;