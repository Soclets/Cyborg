// A step of 0 never ends, so it's a failure, and a `for` has nothing to handle it.
// trace: D145
import std;
@config(default);

fn.main [] -> void:
    let $by: i32 = 0;
    std::io::fmt_print("before{newln}");
    for $i in range[from: 1, to: 5, step_by: $by]:
        std::io::fmt_print("inside{newln}");
    end for
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
