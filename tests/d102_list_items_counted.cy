// trace: D102
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $b: std::list<i32>[len.min: 1 items, len.max: 8 items] set_to [4, 6];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.len);
    for $x in $b:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    end for
    return void;
end fn.main
