// trace: D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns i32:
    return $b.n;
end fn eat

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: box = box[n = 7];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", eat(Copy $b));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.n);
    return void;
end fn.main
