// `from: 1` is past a length of 0, so going through an empty list just skips the loop.
// trace: D145, D142
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = empty;
    for $i in range[from: 1, to: $xs.len]:
        std::io::fmt_print("inside{newln}");
    end for
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
