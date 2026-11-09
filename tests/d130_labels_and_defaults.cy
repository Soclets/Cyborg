// trace: D130, D114
@config(default);
import std;

struct box:
    n: i32;
end struct box

fn box.%plus [$b: box, $by: i32] -> i32:
    labels [$by as by];
    defaults [$by = 1];

    return $b.n + $by;
end fn box.%plus

fn.main [] -> void:
    let $b: box = box[n = 1];
    std::io::fmt_print("{}{newln}", $b.%plus());
    std::io::fmt_print("{}{newln}", $b.%plus(by: 5));
    return void;
end fn.main
