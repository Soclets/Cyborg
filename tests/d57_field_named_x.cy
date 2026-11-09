// `x` is an operator, but a field has no clash, because `.` says what it is.
// trace: D57, D75
import std;
@config(default);

struct point:
    x: i32;
    y: i32;
end struct point

fn.main [] -> void:
    let $p: point = point[x = 3, y = 4];
    let $n: i32 = $p.x x $p.y;
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
