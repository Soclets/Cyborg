// trace: D130, D91
import std;
import os;

struct box:
    n: i32;
end struct box

fn.pure.baked box.%twice [$b: box] returns i32:
    return $b.n x 2;
end fn.pure.baked box.%twice

const $b: box set_to box[n set_to 4];
const $c: i32 set_to $b.%twice();

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $d: i32 set_to bake $b.%twice();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d);
    return void;
end fn.main
