// trace: D130, D120
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: box = box[n = 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.%read());
    return void;
end fn.main

fn box.%read [$b: Lent box] -> i32:
    return $b.n;
end fn box.%read

struct box:
    n: i32;
end struct box
