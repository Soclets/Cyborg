// `Lend.mut` may change the value, so its field can change too. Reported as a reading.
// trace: D125, D2
import std;
@config(default);

struct box:
    n: i32;
end struct box

fn show [$b: box] -> void:
    std::io::fmt_print("{}{newln}", $b.n);
    return void;
end fn show

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    let $l: Lend.mut from $b box = Lend.mut $b;
    change $l.n = 2;
    show(Lend $l);
    return void;
end fn.main
