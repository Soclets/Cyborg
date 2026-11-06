// trace: D113
import std;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> i32:
    labels [$b as val];

    return $b.n;
end fn eat

fn.main [] -> void:
    let $r: box = box[n = 5];
    std::println(eat(val: Give $r) as str);
    let mut $xs: std::list<box>[len.max: inf items] = empty;
    let $q: box = box[n = 6];
    $xs.std::%append(val: Give $q);
    $xs.std::%append(val: box[n = 7]);
    std::println($xs.len as str);
    return void;
end fn.main
