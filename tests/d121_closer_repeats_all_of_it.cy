// trace: D121, D86, D91
import std;
import os;

fn.pure area [$w: i32, $h: i32] -> i32:
    return $w x $h;
end fn.pure area

fn.pure.baked seconds_in [$days: i32] -> i32:
    return $days x 86_400;
end fn.pure.baked seconds_in

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", area(3, 4));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", seconds_in(2));
    return void;
end fn.main
