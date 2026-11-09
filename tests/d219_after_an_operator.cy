// A line can break after a binary operator, `=` included.
// trace: D219
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a +
        2;
    let $c: i32 =
        $b x 3;
    std::io::fmt_print("{}{newln}", $c);
    return void;
end fn.main
