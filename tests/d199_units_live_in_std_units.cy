// trace: D199, D196
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64[unit: std::units::time.seconds.span] = 3600 std::units::time.seconds;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
