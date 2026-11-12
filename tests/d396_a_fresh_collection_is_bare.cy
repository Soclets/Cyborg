// A collection with no holder is made there and gone after the loop, so its loop takes no word,
// like a range.
// trace: D396, D393
import std;
import os;

fn pair [$n: i32] -> std::coll::array<i32>[len: 2 items]:
    return [$n, $n + 1];
end fn pair

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    for $x in pair(4):
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    end for
    return void;
end fn.main
