// The standard's example: lending an item names the whole list.
// trace: D170, D172
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn first [$xs: std::list<reading>[len.min: 1 items, len.max: inf items]] -> Lend from $xs reading:
    return Lend $xs[pos: 1];
end fn first

fn.main [] -> void:
    let $xs: std::list<reading>[len.min: 1 items, len.max: inf items] = [reading[celsius = 3]];
    let $f: Lend from $xs reading = first(Lend $xs);
    std::println($f.celsius as str);
    return void;
end fn.main
