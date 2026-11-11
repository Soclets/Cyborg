// A step of 0 never ends, so it's a failure, and a `for` has nothing to handle it.
// trace: D145
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $by: i32 set_to 0;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    for $i in range[from: 1, to: 5, step_by: $by]:
        std::io::fmt_print(Lend.mut $out, "inside{newln}");
    end for
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
