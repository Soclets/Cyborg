// trace: D114
import std;
import os;

fn pair [$a: i32, $b: i32] -> i32:
    labels.any_order [$a as first, $b as second];

    return $a x 10 + $b;
end fn pair

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", pair(second: 1, first: 2));
    return void;
end fn.main
