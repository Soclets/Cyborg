// trace: D91
import std;
import os;

fn.pure.baked seconds_in [$days: i32] returns i32:
    return $days x 86_400;
end fn.pure.baked seconds_in

const $week: i32 set_to seconds_in(7);

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $day: i32 set_to bake seconds_in(1);
    let $d: i32 set_to 2;
    let $n: i32 set_to seconds_in($d);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $week);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $day);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
