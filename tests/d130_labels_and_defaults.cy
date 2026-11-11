// trace: D130, D114
import std;
import os;

struct box:
    n: i32;
end struct box

fn box.%plus [$b: box, $by: i32] returns i32:
    labels [$by as by];
    defaults [$by = 1];

    return $b.n + $by;
end fn box.%plus

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: box = box[n = 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.%plus());
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.%plus(by: 5));
    return void;
end fn.main
