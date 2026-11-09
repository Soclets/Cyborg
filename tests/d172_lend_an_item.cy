// trace: D172
import std;
@config(default);

struct inner:
    n: i32;
end struct inner

fn.main [] -> void:
    let $xs: std::list<inner>[len.max: inf items] = [inner[n = 1], inner[n = 2]];
    let $first: Lend from $xs inner = Lend $xs[pos: 1];
    std::println($first.n as str);
    return void;
end fn.main
