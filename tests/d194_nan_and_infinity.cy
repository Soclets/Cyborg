// trace: D194, D166
@config(default);
import std;

fn.main [] -> void:
    let $zero: f64.ieee = 0.0;
    std::io::fmt_print("{} {} {}{newln}", 1.0 div $zero, -1.0 div $zero, $zero div $zero);
    return void;
end fn.main
