// The Uno R4 WiFi's Cortex-M4 takes one 32-bit lane.
// trace: D292, D389
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: simd<i32>[lanes: max] = all 7;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.%sum());
    return void;
end fn.main
