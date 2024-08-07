architecture sys2 of top is
  component ent1
    port(
      ok: out bit
    );
  end component;
begin

  u_ent1: ent1
    port map(
      ok => ok
    );
end architecture;