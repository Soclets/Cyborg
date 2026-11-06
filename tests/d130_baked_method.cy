// trace: D130, D91
import std;

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
    std::println($c as str);
    std::println($d as str);
    return void;
end fn.main
