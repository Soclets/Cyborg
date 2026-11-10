// trace: D109, D112
import std;

fam.unit time:
    time.minutes = 60 std::units::time.seconds;
end fam.unit time

fn.main [none] -> void:
    let $m: i32[unit: time.minutes] = 2 time.minutes;
    std::io::fmt_print("{}{newln}", $m);
    return void;
end fn.main
