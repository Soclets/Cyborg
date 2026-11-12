// On the Mac, a vector is 128 bits: 4 `i32`s, 2 `i64`s, 16 `u8`s, 2 `f64`s.
// trace: D292, D389
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: simd<i32>[lanes: max] = all 1;
    let $b: simd<i64>[lanes: max] = all 1;
    let $c: simd<u8>[lanes: max] = all 1;
    let $d: simd<f64.stable>[lanes: max] = all 1.0;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $a.%sum(), $b.%sum(), $c.%sum(), $d.%sum());
    return void;
end fn.main
