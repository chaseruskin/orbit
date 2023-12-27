library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

library std;
use std.textio.all;

library work;
use work.casting;

package drivers is

    -- LOGIC FUNCTIONS

    -- Produces a 50% duty cycle clock 'clk' with a period of 'period' that
    -- is continuously driven until 'halt' is set to true. 
    procedure spin_clock(signal clk: out std_logic; period: time; signal halt: boolean);

    -- Asynchronously applies the reset (active-high) and then synchronously
    -- de-asserts the reset after 'cycles' clock cycles generated by 'clk'.
    --
    -- The reset will not be applied if 'cycles' is set to 0. The reset will
    -- de-assert on the falling edge of the 'cycles' count clock cycle.
    procedure reset_system(signal clk: std_logic; signal rst: out std_logic; cycles: natural);

    -- Drive a logic[] signal 'wire' with a value from the line 'row'.
    procedure drive(variable row: inout line; signal vec: out std_logic_vector);

    -- Drive a logic signal 'wire' with a value from the line 'row'.
    procedure drive(variable row: inout line; signal wire: out std_logic);

    -- Read a logic[] value from the line 'row' into the variable 'ideal'.
    procedure load(variable row: inout line; variable ideal: out std_logic_vector);

    -- Read a logic value from the line 'row' into the variable 'ideal'.
    procedure load(variable row: inout line; variable ideal: out std_logic);

    -- Awaits the flag to be asserted or until delay time is up. Sets 'timeout'
    -- to true when the flag was asserted before the timeout and false if
    -- the timeout was reached before the flag was raised.
    --
    -- Sets an unbounded waiting limit if 'cycles' is set to 0.
    procedure monitor(signal clk: std_logic; signal flag: std_logic; cycles: natural; variable timeout: out boolean);
    
    -- Sets 'halt' to true, prints a message to the console, and enters an
    -- infinite wait statement to signal that the simulation is complete.
    procedure complete(signal halt: out boolean);

    -- Enters an infinite wait if the 'halt' signal is set to true.
    procedure check(halt: in boolean);

    -- LOGGING FUNCTIONS

    type log_level is (TRACE, DEBUG, INFO, WARN, ERROR, FATAL);

    -- Captures any event during simulation and writes the outcome record to the file 'fd'.
    --
    -- The time when the procedure is called is recorded in the timestamp.
    procedure log_event(file fd: text; sev: log_level; topic: string; cause: string);
    
    procedure log_monitor(file fd: text; signal clk: std_logic; signal flag: std_logic; cycles: natural; variable timeout: out boolean; cause: string);
    
    procedure log_assertion(file fd: text; received: std_logic; expected: std_logic; cause: string);
    procedure log_assertion(file fd: text; received: std_logic_vector; expected: std_logic_vector; cause: string);
    
    procedure log_stability(file fd: text; signal clk: std_logic; signal cond: std_logic; signal vec: std_logic_vector; cause: string);

end package;


package body drivers is

    -- LOGIC FUNCTIONS

    procedure complete(signal halt: out boolean) is
    begin
        -- report "Simulation complete";
        halt <= true;
        wait;
    end procedure;


    procedure check(halt: in boolean) is
    begin
        if halt = true then 
            wait;
        end if;
    end procedure;


    procedure spin_clock(signal clk: out std_logic; period: time; signal halt: boolean) is
        variable inner_clk : std_logic := '0';
    begin
        while halt = false loop
            clk <= inner_clk;
            wait for period/2;
            inner_clk := not inner_clk;
        end loop;
        wait;
    end procedure;


    procedure reset_system(signal clk: std_logic; signal rst: out std_logic; cycles: natural) is
    begin
        -- only apply the reset if the number of cycles to delay is greater than zero
        if cycles > 0 then
            rst <= '1';
            wait for 0 ns;
            for delay in 1 to cycles loop
                wait until rising_edge(clk);
            end loop;
            wait until falling_edge(clk);
        end if;
        rst <= '0';
        -- wait for 0 ns;
    end procedure;


    procedure monitor(signal clk: std_logic; signal flag: std_logic; cycles: natural; variable timeout: out boolean) is
        variable cycle_count : natural := 0;
    begin
        timeout := true;
        if cycles = 0 then
            wait until rising_edge(clk) and flag = '1';
            timeout := false;
        else
            while cycle_count < cycles loop
                if flag = '1' then
                    timeout := false;
                    exit;
                end if;
                wait until rising_edge(clk);
                cycle_count := cycle_count + 1;
            end loop;
        end if;
    end procedure;


    procedure drive(variable row: inout line; signal vec: out std_logic_vector) is
        variable word      : string(vec'range);
        variable temp      : std_logic_vector(vec'range);
        variable delimiter : character;
    begin
        if row'length > 0 then
            read(row, word);
            for ii in vec'range loop
                temp(ii) := casting.char_to_logic(word(ii));
            end loop;
            vec <= temp;
            -- ignore the delimiter
            read(row, delimiter);
            -- wait for 0 ns;
        end if;
    end procedure;


    procedure load(variable row: inout line; variable ideal: out std_logic_vector) is
        variable word      : string(ideal'range);
        variable delimiter : character;
    begin
        if row'length > 0 then
            read(row, word);
            for ii in ideal'range loop
                ideal(ii) := casting.char_to_logic(word(ii));
            end loop;
            -- ignore the delimiter
            read(row, delimiter);
        end if;
    end procedure;


    procedure drive(variable row: inout line; signal wire: out std_logic) is
        variable word      : character;
        variable delimiter : character;
    begin
        if row'length > 0 then
            read(row, word);
            wire <= casting.char_to_logic(word);
            -- ignore the delimiter
            read(row, delimiter);
            -- wait for 0 ns;
        end if;
    end procedure;


    procedure load(variable row: inout line; variable ideal: out std_logic) is
        variable word      : character;
        variable delimiter : character;
    begin
        if row'length > 0 then
            read(row, word);
            ideal := casting.char_to_logic(word);
            -- ignore the delimiter
            read(row, delimiter);
        end if;
    end procedure;


    -- LOGGING FUNCTIONS

    procedure log_event(file fd: text; sev: log_level; topic: string; cause: string) is
        variable row : line;
        variable topic_filtered : string(topic'range);
        constant TIMESTAMP_SHIFT : positive := 15;
        constant LOGLEVEL_SHIFT : positive := 8;
        constant TOPIC_SHIFT : positive := 12;
    begin
        -- write the timestamp ("when")
        write(row, '[');
        write(row, now, left, TIMESTAMP_SHIFT);
        write(row, ']');
        write(row, ' ');

        -- write the log level ("why") ... TRACE, DEBUG, INFO, WARN, ERROR, FATAL
        if sev = TRACE then
            write(row, string'("TRACE"), left, LOGLEVEL_SHIFT);
        elsif sev = DEBUG then
            write(row, string'("DEBUG"), left, LOGLEVEL_SHIFT);
        elsif sev = INFO then
            write(row, string'("INFO"), left, LOGLEVEL_SHIFT);
        elsif sev = WARN then
            write(row, string'("WARN"), left, LOGLEVEL_SHIFT);
        elsif sev = ERROR then
            write(row, string'("ERROR"), left, LOGLEVEL_SHIFT);
        elsif sev = FATAL then
            write(row, string'("FATAL"), left, LOGLEVEL_SHIFT);
        else
            write(row, string'("INFO"), left, LOGLEVEL_SHIFT);
        end if;

        -- write the topic ("what")
        write(row, ' ');
        -- filter the topic to prevent illegal characters from messing up format
        topic_filtered := topic;
        for ii in topic'range loop
            if topic(ii) = '"' then
                topic_filtered(ii) := '_';
            elsif topic(ii) = ' ' then
                topic_filtered(ii) := '_';
            end if;
        end loop;
        write(row, topic_filtered, left, TOPIC_SHIFT);
        write(row, ' ');

        -- write the root cause ("how")
        write(row, '"');
        write(row, cause);
        write(row, '"');
        writeline(fd, row);
    end procedure;


    procedure log_monitor(file fd: text; signal clk: std_logic; signal flag: std_logic; cycles: natural; variable timeout : out boolean; cause: string) is
        variable cycle_count : natural := 0;
        constant cycle_limit : natural := cycles + 1;
    begin
        timeout := true;
        -- wait forever if there is no clock cycle limit
        if cycle_limit = 0 then
            wait until falling_edge(clk) and flag = '1';
            timeout := false;
            return;
        else
            -- wonky way to count cycles and evaluate on first edge of flag being asserted...
            -- maybe break monitor into 2 separate processes (a cycle counter and a rising flag detector)
            while cycle_count < cycle_limit loop
                if flag = '1' then
                    timeout := false;
                    log_event(fd, INFO, "MONITOR", cause & " - required " & integer'image(cycle_count) & " cycles");
                    return;
                end if;
                -- necessary ordering to escape at correct time in simulation
                cycle_count := cycle_count + 1;
                if cycle_count < cycle_limit then
                    wait until falling_edge(clk);
                end if;
            end loop;
        end if;
        -- reached this point, then a violation has occurred
        log_event(fd, ERROR, "MONITOR", cause & " - never asserted after waiting " & integer'image(cycles) & " cycles");
    end procedure;


    procedure log_assertion(file fd: text; received: std_logic; expected: std_logic; cause: string) is
    begin
        if received /= expected then
            log_event(fd, ERROR, "ASSERTION", cause & " - received " & casting.to_str(received) & " does not match expected " & casting.to_str(expected));
        else 
            log_event(fd, INFO, "ASSERTION", cause & " - received " & casting.to_str(received) & " matches expected " & casting.to_str(expected));
        end if;
    end procedure;


    procedure log_assertion(file fd: text; received: std_logic_vector; expected: std_logic_vector; cause: string) is
    begin
        if received /= expected then
            log_event(fd, ERROR, "ASSERTION", cause & " - received " & casting.to_str(received) & " does not match expected " & casting.to_str(expected));
        else 
            log_event(fd, INFO, "ASSERTION", cause & " - received " & casting.to_str(received) & " matches expected " & casting.to_str(expected));
        end if;
    end procedure;


    procedure log_stability(file fd: text; signal clk: std_logic; signal cond: std_logic; signal vec: std_logic_vector; cause: string) is
        variable vec_prev: std_logic_vector(vec'range);
        variable is_okay: boolean := true;
    begin
        wait until rising_edge(cond);
        -- wait until rising_edge(cond);
        vec_prev := vec;
        while cond = '1' loop
            -- check if its been stable since the rising edge of done
            if vec_prev /= vec then
                is_okay := false;
                log_event(fd, ERROR, "STABILITY", cause & " - updated value " & casting.to_str(vec) & " lost stability of " & casting.to_str(vec_prev));
            end if;

            wait until rising_edge(clk);
        end loop;
            if is_okay = true then
                log_event(fd, INFO, "STABILITY", cause & " - maintained stability at " & casting.to_str(vec_prev));
            end if;
    end procedure;


end package body;