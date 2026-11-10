// A pure function changes nothing outside itself, and its own variable isn't outside.
// trace: D125, D86
import std;

struct box:
    n: i32;
end struct box

fn.pure bumped [$start: i32] -> i32:
    let mut $b: box = box[n = $start];
    change $b.n = $b.n + 1;
    return $b.n;
end fn.pure bumped

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", bumped(1));
    return void;
end fn.main
