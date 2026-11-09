// The second time round, it's already gone.
// trace: D2
import std;
@config(default);

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    let $b: box = box[n = 1];
    for $x in $xs:
        eat(Give $b);
    end for
    return void;
end fn.main
