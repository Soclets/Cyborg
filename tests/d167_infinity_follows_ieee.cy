// trace: D167, D166
@config(default);
import std;

fn.main [] -> void:
    let $zero: f64.ieee = 0.0;
    let $inf: f64.ieee = 1.0 div $zero;
    let $down: f64.ieee = -1.0 div $zero;
    std::io::fmt_print("{}{newln}", ($inf + 1.0));
    std::io::fmt_print("{}{newln}", ($inf x -2.0));
    std::io::fmt_print("{}{newln}", ($down - $inf));
    std::io::fmt_print("{}{newln}", ($inf + $down));
    std::io::fmt_print("{}{newln}", (1.0 div $inf));
    if $down < $inf:
        std::io::fmt_print("ordered{newln}");
    end if
    return void;
end fn.main
