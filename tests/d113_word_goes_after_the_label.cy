// trace: D113
import std;
import os;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> i32:
    labels [$b as val];

    return $b.n;
end fn eat

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: box = box[n = 5];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", eat(val: Give $r));
    let mut $xs: std::list<box>[len.max: inf items] = empty;
    let $q: box = box[n = 6];
    $xs.std::%%append(val: Give $q);
    $xs.std::%%append(val: box[n = 7]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
