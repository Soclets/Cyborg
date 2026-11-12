// Adding up the lanes goes in pairs, with an odd lane carried up: `(a + b) + c`. So `[1,
// 2147483647, -1]` overflows at the first pair on every CPU, where `a + (b + c)` wouldn't.
// trace: D296, D297
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: simd<f64.stable>[lanes: 3] = [~0.1, ~0.2, ~0.3];
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $a.%sum());
    let $b: simd<i32>[lanes: 3] = [1, 2147483647, -1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.%sum());
    return void;
end fn.main
