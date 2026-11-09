// trace: D133, D125, D57
import std;
@config(default);

struct box:
    n: i32;
end struct box

struct bag:
    boxes: std::array<box>[len: 2 items];
end struct bag

fn box.%bump [$b: Lent.mut box] -> void:
    change $b.n = $b.n + 1;
    return void;
end fn box.%bump

fn.main [] -> void:
    let mut $b: bag = bag[boxes = [box[n = 1], box[n = 2]]];
    std::println($b.boxes[pos: 2].n as str);
    change $b.boxes[pos: 1].n = 5;
    std::println($b.boxes[offset.items: 0].n as str);
    $b.boxes[pos: 1].%bump();
    std::println($b.boxes[pos: 1].n as str);
    change $b.boxes[pos: 2] = box[n = 9];
    std::println($b.boxes[pos: 2].n as str);
    return void;
end fn.main
