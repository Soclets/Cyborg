// Only running shows these numbers, and a range that can't reach its end runs zero rounds.
// trace: D145
@config(default);
import std;

fn.main [] -> void:
    let $start: i32 = 5;
    std::io::fmt_print("before{newln}");
    for $i in range[from: $start, to: 1]:
        std::io::fmt_print("inside{newln}");
    end for
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
