// `Lend.mut` may change the value, so its field can change too. Reported as a reading.
// trace: D125, D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn show [$b: Lent box] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.n);
    return void;
end fn show

fn.main [none] -> void:
    let mut $b: box = box[n = 1];
    let $l: Lend.mut from $b box = Lend.mut $b;
    change $l.n = 2;
    show(Lend $l);
    return void;
end fn.main
