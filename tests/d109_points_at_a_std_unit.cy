// trace: D109, D112
import std;
import os;

fam.unit time:
    time.minutes set_to 60 std::units::time.seconds;
end fam.unit time

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $m: i32[unit: time.minutes] set_to 2 time.minutes;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m);
    return void;
end fn.main
