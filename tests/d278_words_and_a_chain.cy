// `below`, `above`, `at_least` and `at_most`, and a chain, which reads `1 below $x below 5`.
// trace: D278, D117
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: i32 set_to 3;
    if $x below 4:
        std::io::fmt_print(Lend.mut $out, "below 4{newln}");
    end if
    if $x above 4:
        std::io::fmt_print(Lend.mut $out, "above 4{newln}");
    end if
    if 1 below $x below 5:
        std::io::fmt_print(Lend.mut $out, "between 1 and 5{newln}");
    end if
    if 4 below $x below 5:
        std::io::fmt_print(Lend.mut $out, "between 4 and 5{newln}");
    end if
    if $x at_least 3 and $x at_most 3:
        std::io::fmt_print(Lend.mut $out, "exactly 3{newln}");
    end if
    return void;
end fn.main
