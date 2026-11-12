// Walking in `simd` chunks says what happens to the leftovers: `.pad` fills the last chunk,
// `.leftovers` hands them to a section one at a time, and `.%%simd_chunks.mut` writes back.
// trace: D299, D300, D303, D358
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $total: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.std::%simd_chunks.pad(lanes: 4, pad: 0):
        change $total = $total + $chunk;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $total.%sum());
    let mut $rest: i32 = 0;
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.std::%simd_chunks.leftovers(lanes: 4):
        change $sum = $sum + $chunk;
    leftovers $x:
        change $rest = $rest + $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $sum.%sum(), $rest);
    for $chunk in $xs.std::%%simd_chunks.mut.leftovers(lanes: 4):
        change $chunk = $chunk + $chunk;
    leftovers $x:
        change $x = 0;
    end for
    for $x in $xs:
        std::io::fmt_print(Lend.mut $out, "{$x} ");
    end for
    std::io::fmt_print(Lend.mut $out, "{newln}");
    return void;
end fn.main
