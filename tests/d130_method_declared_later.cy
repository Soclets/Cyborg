// trace: D130, D120
import std;
@config(default);

fn.main [] -> void:
    let $b: box = box[n = 1];
    std::println($b.%read() as str);
    return void;
end fn.main

fn box.%read [$b: box] -> i32:
    return $b.n;
end fn box.%read

struct box:
    n: i32;
end struct box
