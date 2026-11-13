// Times three integer workloads on the Uno R4 WiFi's Cortex-M4 with its cycle counter,
// DWT CYCCNT, and leaves each answer and its cycle count in RAM from 0x2000_0000, where a debugger
// reads them. No serial printing exists yet (D252's second milestone), and no arrays, since
// today's compiler can need `memset` for those, which its bare-metal runtime lacks.
// No `module` line yet: the compiler is holding modules (D412 to D415) until the tests have them.
// trace: D252, D253, D317

fn collatz_steps [$limit: i64] -> i64:
    let mut $total: i64 = 0;
    let mut $n: i64 = 1;
    while $n at_most $limit:
        let mut $x: i64 = $n;
        while $x != 1:
            if $x mod.trunc 2 == 0:
                change $x = $x div.trunc 2;
            else:
                change $x = 3 x $x + 1;
            end if
            change $total = $total + 1;
        end while
        change $n = $n + 1;
    end while
    return $total;
end fn collatz_steps

fn primes_below [$limit: i32] -> i32:
    let mut $count: i32 = 0;
    let mut $n: i32 = 2;
    while $n below $limit:
        let mut $d: i32 = 2;
        let mut $prime: bool = true;
        while $d x $d at_most $n and $prime:
            if $n mod.trunc $d == 0:
                change $prime = false;
            end if
            change $d = $d + 1;
        end while
        if $prime:
            change $count = $count + 1;
        end if
        change $n = $n + 1;
    end while
    return $count;
end fn primes_below

fn xorshift [$rounds: i32] -> u32:
    let mut $x: u32 = 2463534242;
    let mut $i: i32 = 0;
    while $i below $rounds:
        // The bits that would fall off are masked off first, which gives the same answer, since
        // today's compiler wrongly treats a `shift.left` that drops bits as overflow (3.7).
        change $x = $x xor.bit (($x and.bit hex 7_ffff) shift.left 13);
        change $x = $x xor.bit ($x shift.right 17);
        change $x = $x xor.bit (($x and.bit hex 7ff_ffff) shift.left 5);
        change $i = $i + 1;
    end while
    return $x;
end fn xorshift

fn.main [none] -> void:
    shield bench:
        reason "timing needs the Cortex-M4's debug cycle counter, and results go to fixed RAM"
        // DEMCR's TRCENA turns the trace block on, and DWT_CTRL's CYCCNTENA starts the counter.
        let $demcr: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex e000_edfc);
        let $dwt_control: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex e000_1000);
        let $cycles: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex e000_1004);
        $demcr.%write(val: $demcr.%read() or.bit hex 100_0000);
        $dwt_control.%write(val: $dwt_control.%read() or.bit hex 1);

        // 0x2000_0000 is a marker: 0 while running, then hex c7b0_0001 when every result is in.
        let $done: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: hex 2000_0000);
        $done.%write(val: hex 0);

        let $start_1: u32 = $cycles.%read();
        let $steps: i64 = collatz_steps(30000);
        let $end_1: u32 = $cycles.%read();
        ptr.volatile.mut<i64>.at(address: hex 2000_0008).%write(val: $steps);
        ptr.volatile.mut<u32>.at(address: hex 2000_0010).%write(val: $end_1 minus.wrap $start_1);

        let $start_2: u32 = $cycles.%read();
        let $primes: i32 = primes_below(20000);
        let $end_2: u32 = $cycles.%read();
        ptr.volatile.mut<i32>.at(address: hex 2000_0014).%write(val: $primes);
        ptr.volatile.mut<u32>.at(address: hex 2000_0018).%write(val: $end_2 minus.wrap $start_2);

        let $start_3: u32 = $cycles.%read();
        let $random: u32 = xorshift(1000000);
        let $end_3: u32 = $cycles.%read();
        ptr.volatile.mut<u32>.at(address: hex 2000_001c).%write(val: $random);
        ptr.volatile.mut<u32>.at(address: hex 2000_0020).%write(val: $end_3 minus.wrap $start_3);

        $done.%write(val: hex c7b0_0001);
        forever:
            let $idle: u32 = $done.%read();
        end forever
    end shield bench
    return void;
end fn.main
