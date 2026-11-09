// trace: D216
@config(default);
import std;

fn.main [] -> void:
    let $total: i32 = 41;
    let mut $next := $total + 1;
    change $next = $next x 2;
    let $name := "rope";
    let $ok := $total > 3;
    std::io::fmt_print("{} {} {}{newln}", $next, $name, $total);
    return void;
end fn.main
