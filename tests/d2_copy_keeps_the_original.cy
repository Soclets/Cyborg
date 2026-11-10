// trace: D2
import std;

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> i32:
    return $b.n;
end fn eat

fn.main [none] -> void:
    let $b: box = box[n = 7];
    std::io::fmt_print("{}{newln}", eat(Copy $b));
    std::io::fmt_print("{}{newln}", $b.n);
    return void;
end fn.main
