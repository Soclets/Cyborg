// trace: D130, D115
import std;

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    $b.std::%read();
    return void;
end fn.main
