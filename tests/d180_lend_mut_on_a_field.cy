// trace: D180
import std;

struct holder:
    items: std::array<i32>[len: 2 items];
end struct holder

fn.main [none] returns void:
    let mut $h: holder set_to holder[items set_to [1, 2]];
    for $n in Lend.mut $h.items:
        change $n set_to 0;
    end for
    return void;
end fn.main
