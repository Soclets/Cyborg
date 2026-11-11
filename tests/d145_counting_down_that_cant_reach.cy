// trace: D145
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $start: i32 set_to 1;
    for $i in range[from: $start, to: 5, step_by: -1]:
        std::io::fmt_print(Lend.mut $out, "inside{newln}");
    end for
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
