-- Some random comments
-- are here!

library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

library work;
use work.hamm_pkg.all;

-- Generic hamming-code encoder that takes a message `message` and packages
-- it with corresponding parity bits into an `encoding` for extended 
-- hamming code (SECDED).
--
-- Implemented in purely combinational logic. Parity bits are set in the
-- indices corresponding to powers of 2 (0, 1, 2, 4, 8, ...).
--
-- <p style="color: red; font-family: monospace;">This text is red.</p>
entity hamm_enc is 
    generic (
        -- Number of parity bits to encode (excluding 0th DED bit)
        PARITY_BITS : positive range 2 to positive'high 
    );
    port (
        -- Incoming message
        message  : in  logics(data_size(PARITY_BITS)-1 downto 0);
        -- Outgoing encoding
        encoding : out logics(block_size(PARITY_BITS)-1 downto 0)
    );
end entity hamm_enc;

-- My basic architecture.
architecture rtl of hamm_enc is
    constant EVEN_PARITY : boolean := true;

    constant TOTAL_BITS_SIZE  : positive := block_size(PARITY_BITS);
    constant PARITY_LINE_SIZE : positive := TOTAL_BITS_SIZE/2;

    type hamm_block is array (0 to PARITY_BITS-1) of logics(PARITY_LINE_SIZE-1 downto 0);

    signal enc_block : hamm_block;

    -- +1 parity for the zero-th check bit
    signal check_bits : logics(PARITY_BITS-1+1 downto 0);

    signal empty_block : logics(TOTAL_BITS_SIZE-1 downto 0);
    signal full_block  : logics(TOTAL_BITS_SIZE-1 downto 0);

begin
    --! Formats the incoming message into a clean hamming-code block with parity
    --! bits cleared.
    process(message)
        variable ctr : natural;
    begin
        empty_block <= (others => '0');
        ctr := 0;
        for ii in 0 to TOTAL_BITS_SIZE-1 loop
            -- use information bit otherwise reserve for parity bit
            if is_pow_2(ii) = false then
                empty_block(ii) <= message(ctr);
                ctr := ctr + 1;
            end if;
        end loop;
    end process;

    --! divide the entire hamming-code block into parity subset groups
    process(empty_block)
        variable temp_line : logics(PARITY_LINE_SIZE-1 downto 0);
        variable index     : logics(PARITY_BITS-1 downto 0);
    begin
        for ii in PARITY_BITS-1 downto 0 loop
            temp_line := (others => '0');
            for jj in TOTAL_BITS_SIZE-1 downto 0 loop 
                -- decode the parity bit index
                index := (others => '0');
                index := logics(to_unsigned(jj, PARITY_BITS));

                if index(ii) = '1' then 
                    -- insert new bit
                    temp_line := temp_line(PARITY_LINE_SIZE-2 downto 0) & empty_block(jj);
                end if;
            end loop;
            -- drive the ii'th vector in the block as this parity's subset of bits
            enc_block(ii) <= temp_line;
        end loop;
    end process;

    --! instantiate parity checkers for the subset of bits to evaluate
    gen_check_bits: for ii in 0 to PARITY_BITS-1 generate
        u_par : entity work.parity
        generic map (
            SIZE        => TOTAL_BITS_SIZE/2,
            EVEN_PARITY => EVEN_PARITY
        ) port map (
            data      => enc_block(ii),
            check_bit => check_bits(ii)
        );
    end generate gen_check_bits;

    --! fill the hamming-code block with computed parity bits
    process(empty_block, check_bits)
        variable ctr : natural;
    begin
        full_block <= empty_block;
        ctr := 0;

        for ii in 1 to TOTAL_BITS_SIZE-1 loop
            -- use information bit otherwise reserve for parity bit
            if is_pow_2(ii) = true then
                full_block(ii) <= check_bits(ctr);
                ctr := ctr + 1;
            end if;
        end loop;
    end process;

    --! computes the extra parity bit (0th bit) for double-error detection
    u_ded : entity work.parity
    generic map (
        SIZE        => TOTAL_BITS_SIZE-1,
        EVEN_PARITY => EVEN_PARITY
    ) port map (
        data      => full_block(TOTAL_BITS_SIZE-1 downto 1),
        check_bit => check_bits(PARITY_BITS)
    );

    -- drive the output with the hamming-code block and the 0th parity bit 
    encoding <= full_block(TOTAL_BITS_SIZE-1 downto 1) & check_bits(PARITY_BITS);

end architecture rtl;