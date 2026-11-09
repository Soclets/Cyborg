// trace: D130, D114
import std;
@config(default);

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
    std::println($b.%plus() as str);
    std::println($b.%plus(by: 5) as str);
    return void;
end fn.main
