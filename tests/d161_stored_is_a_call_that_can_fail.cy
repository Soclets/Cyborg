// trace: D161, D95
@config(default);
import std;

fn.pure p [$x: f64.stable] -> f64.stable:
    std::io::fmt_print.stored("{}{newln}", $x);
    return $x;
end fn.pure p

fn.main [] -> void:
    return void;
end fn.main
