// trace: D216
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $total: i32 = 41;
    let mut $next := $total + 1;
    change $next = $next x 2;
    let $name := "rope";
    let $ok := $total > 3;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $next, $name, $total);
    return void;
end fn.main
