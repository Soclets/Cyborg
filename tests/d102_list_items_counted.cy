// trace: D102
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: std::coll::list<i32>[len.min: 1 items, len.max: 8 items] = [4, 6];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.len);
    for $x in $b:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    end for
    return void;
end fn.main
