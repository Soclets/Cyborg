// trace: D99, D112, D109
import std;
import os;

fam.unit time:
    time.seconds: base;
    time.minutes = 60 time.seconds;
end fam.unit time

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32[unit: time.seconds] = 3600 time.seconds;
    let $b: i32[unit: time.seconds] = 5 time.seconds;
    let $c: i32[unit: time.seconds] = $a + $b;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    if $c above $a:
        std::io::fmt_print(Lend.mut $out, "more{newln}");
    end if
    return void;
end fn.main
