// trace: D194, D166
import std;
@config(default);

fn.main [] -> void:
    let $zero: f64.ieee = 0.0;
    std::io::fmt_print("{} {} {}{newln}", 1.0 div $zero, -1.0 div $zero, $zero div $zero);
    return void;
end fn.main
