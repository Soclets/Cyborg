// Each round has its own `$x`.
// trace: D203, D128
import std;
@config(default);

fn.main [] -> void:
    for $i in range[from: 1, to: 3]:
        decl $x: i32;
        assign $x = $i x 10;
        std::io::fmt_print("{}{newln}", $x);
    end for
    return void;
end fn.main
