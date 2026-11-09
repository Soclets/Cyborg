// trace: D145
import std;
@config(default);

fn.main [] -> void:
    let $start: i32 = 1;
    for $i in range[from: $start, to: 5, step_by: -1]:
        std::io::fmt_print("inside{newln}");
    end for
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
