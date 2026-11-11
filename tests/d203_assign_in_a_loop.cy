// Going round again would assign it a second time.
// trace: D203, D128
import std;

fn.main [none] -> void:
    decl $x: i32;
    for $i in range[from: 1, to: 3]:
        assign $x = $i;
    end for
    return void;
end fn.main
