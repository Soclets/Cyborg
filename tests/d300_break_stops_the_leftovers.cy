// `break` stops the whole loop, leftovers included.
// trace: D300
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 7 items] = [1, 2, 3, 4, 5, 6, 7];
    for $chunk in $xs.%simd_chunks.leftovers(lanes: 2):
        std::io::fmt_print(Lend.mut $out, "chunk {}{newln}", $chunk.%sum());
    leftovers $x:
        std::io::fmt_print(Lend.mut $out, "leftover {$x}{newln}");
    end for
    for $chunk in $xs.%simd_chunks.leftovers(lanes: 2):
        std::io::fmt_print(Lend.mut $out, "chunk {}{newln}", $chunk.%sum());
        if $chunk.%sum() above 4:
            break;
        end if
    leftovers $x:
        std::io::fmt_print(Lend.mut $out, "leftover {$x}{newln}");
    end for
    return void;
end fn.main
