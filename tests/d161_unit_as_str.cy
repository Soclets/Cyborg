// trace: D161, D122
import std;
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $t: f64.time.seconds = 1.5 time.seconds;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
