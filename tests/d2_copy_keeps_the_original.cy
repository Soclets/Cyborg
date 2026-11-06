// trace: D2
import std;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> i32:
    return $b.n;
end fn eat

fn.main [] -> void:
    let $b: box = box[n = 7];
    std::println(eat(Copy $b) as str);
    std::println($b.n as str);
    return void;
end fn.main
