-- --------------?
entity iso_8859_1 is
end entity iso_8859_1;

architecture char of iso_8859_1 is
begin
	assert_char: assert true report "Sig" & '?' & "si";
end architecture char;

architecture string of iso_8859_1 is
begin
	assert_string: assert true report "Sig?si";
end architecture string;

architecture extended of iso_8859_1 is
	constant \Dreik?sehoch\ : integer := 0;
begin
	assert \Dreik?sehoch\ = 0;
end architecture extended;

architecture comment of iso_8859_1 is
begin
	-- Dies ist ein sch?ner Kommentar
	-- No Greek or Japanese possible here
end architecture comment;