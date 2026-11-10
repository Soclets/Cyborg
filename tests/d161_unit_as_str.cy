// trace: D161, D122
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $t: f64.stable[unit: time.seconds] = 1.5 time.seconds;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
