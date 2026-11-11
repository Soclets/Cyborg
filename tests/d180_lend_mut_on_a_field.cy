// trace: D180
import std;

struct holder:
    items: std::array<i32>[len: 2 items];
end struct holder

fn.main [none] -> void:
    let mut $h: holder = holder[items = [1, 2]];
    for $n in Lend.mut $h.items:
        change $n = 0;
    end for
    return void;
end fn.main
