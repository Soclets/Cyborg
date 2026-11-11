// Only running shows these numbers, and a range that can't reach its end runs zero rounds.
// trace: D145
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $start: i32 = 5;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    for $i in range[from: $start, to: 1]:
        std::io::fmt_print(Lend.mut $out, "inside{newln}");
    end for
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
