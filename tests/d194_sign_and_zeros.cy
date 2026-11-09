// trace: D194
@config(default);
import std;

fn.main [] -> void:
    let $n: i32 = -42;
    std::io::fmt_print("{sign} {sign} {zeros, width: 6 cols} {sign, zeros, width: 6 cols}{newln}", 7, $n, $n, 7);
    return void;
end fn.main
