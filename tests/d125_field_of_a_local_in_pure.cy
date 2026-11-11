// A pure function changes nothing outside itself, and its own variable isn't outside.
// trace: D125, D86
import std;
import os;

struct box:
    n: i32;
end struct box

fn.pure bumped [$start: i32] returns i32:
    let mut $b: box set_to box[n set_to $start];
    change $b.n set_to $b.n + 1;
    return $b.n;
end fn.pure bumped

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", bumped(1));
    return void;
end fn.main
