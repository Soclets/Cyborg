// trace: D130, D91
import std;
@config(default);

struct box:
    n: i32;
end struct box

fn.pure.baked box.%twice [$b: box] -> i32:
    return $b.n x 2;
end fn.pure.baked box.%twice

const $b: box = box[n = 4];
const $c: i32 = $b.%twice();

fn.main [] -> void:
    let $d: i32 = bake $b.%twice();
    std::io::fmt_print("{}{newln}", $c);
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
