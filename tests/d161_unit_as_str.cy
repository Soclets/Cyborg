// trace: D161, D122
@config(default);
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $t: f64.time.seconds = 1.5 time.seconds;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
