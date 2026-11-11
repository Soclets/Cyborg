// trace: D130, D115
import std;

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $b: box = box[n = 1];
    $b.std::%read();
    return void;
end fn.main
