library ieee;
use ieee.std_logic_1164.all;

entity t1 is 
    port (
        rx  : in std_logic;
        tx  : out std_logic;
        valid : out std_logic
    );
end entity;

architecture rtl of t1 is
begin
    tx <= not rx;
    valid <= '1';
end architecture;

library ieee;
use ieee.std_logic_1164.all;
use std.textio.all;

library lib;
use lib.drivers.all;

entity t1_tb is
end entity;

architecture bench of t1_tb is

    constant WIDTH_A : positive := 12;

    --! internal test signals
    signal slv0 : std_logic_vector(WIDTH_A-1 downto 0) := (others => '0');
    signal slv1 : std_logic_vector(WIDTH_A-1 downto 0) := (others => '0');
    signal slv2 : std_logic_vector(7 downto 0) := (others => '0');
    signal sl0 : std_logic;

    signal rx : std_logic := '0';
    signal tx : std_logic := '0';
    signal valid : std_logic := '0';

    constant TIMEOUT_LIMIT : natural := 1000;

    -- always include these signals in a testbench
    signal clk   : std_logic;
    signal reset : std_logic;
    signal halt  : boolean := false;

    file events : text open write_mode is "events.log";

begin
    -- unit-under-test
    DUT : entity work.t1 
        port map(
            tx => tx,
            rx => rx,
            valid => valid
        );

    --! Generate a 50% duty cycle for 25 Mhz
    spin_clock(clk, 40 ns, halt);

    --! test reading a file filled with test vectors
    PRODUCER : process
        file inputs : text open read_mode is "inputs.trace";

        -- @note: auto-generate procedure from python script because that is where
        -- order is defined for test vector inputs
        procedure drive_transaction(file fd: text) is 
            variable row : line;
        begin
            if endfile(fd) = false then
                -- drive a transaction
                readline(fd, row);
                drive(row, slv0);
                drive(row, slv1);
                drive(row, sl0);
                drive(row, rx);
            end if;
        end procedure;

    begin  
        -- initialize input signals      
        drive_transaction(inputs);
        reset_system(clk, reset, 3);
        wait until rising_edge(clk);

        -- drive transactions
        while endfile(inputs) = false loop
            drive_transaction(inputs);
            wait until rising_edge(clk);
        end loop;

        -- wait for all outputs to be checked
        wait;
    end process;

    CONSUMER : process
        file outputs : text open read_mode is "outputs.trace";
        variable timeout : boolean;

        -- @note: auto-generate procedure from python script because that is where
        -- order is defined for test vector outputs
        procedure score_transaction(file fd: text; file ld: text) is
            variable row : line;
            variable ideal_tx : std_logic;
        begin
            if endfile(fd) = false then
                -- compare expected outputs and inputs
                readline(fd, row);
                load(row, ideal_tx);
                log_assertion(ld, tx, ideal_tx, "tx");
            end if;
        end procedure;

    begin
        wait until reset = '0';

        while endfile(outputs) = false loop
            -- wait for a valid time to check
            log_monitor(events, clk, valid, TIMEOUT_LIMIT, timeout, "valid");
            -- compare outputs
            score_transaction(outputs, events);
            wait until rising_edge(clk);
        end loop;

        -- use a custom log record (demonstrates filtering of topic too)
        log_event(events, TRACE, "SOME EVENT", "manual capture");
        
        -- force an ERROR assertion into log
        log_assertion(events, valid, '1', "valid");

        -- halt the simulation
        complete(halt);
    end process;

end architecture;