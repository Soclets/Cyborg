// trace: D113
import std;
import os;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns i32:
    labels [$b as val];

    return $b.n;
end fn eat

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: box set_to box[n set_to 5];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", eat(val: Give $r));
    let mut $xs: std::list<box>[len.max: inf items] set_to empty;
    let $q: box set_to box[n set_to 6];
    $xs.std::%%append(val: Give $q);
    $xs.std::%%append(val: box[n set_to 7]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
