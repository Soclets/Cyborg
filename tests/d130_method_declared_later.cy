// trace: D130, D120
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $b: box set_to box[n set_to 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.%read());
    return void;
end fn.main

fn box.%read [$b: box] returns i32:
    return $b.n;
end fn box.%read

struct box:
    n: i32;
end struct box
