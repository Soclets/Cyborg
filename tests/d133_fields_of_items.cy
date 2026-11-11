// trace: D133, D125, D57
import std;
import os;

struct box:
    n: i32;
end struct box

struct bag:
    boxes: std::array<box>[len: 2 items];
end struct bag

fn box.%%bump [$b: Lent.mut box] -> void:
    change $b.n = $b.n + 1;
    return void;
end fn box.%%bump

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $b: bag = bag[boxes = [box[n = 1], box[n = 2]]];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.boxes[pos: 2].n);
    change $b.boxes[pos: 1].n = 5;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.boxes[offset.items: 0].n);
    $b.boxes[pos: 1].%%bump();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.boxes[pos: 1].n);
    change $b.boxes[pos: 2] = box[n = 9];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.boxes[pos: 2].n);
    return void;
end fn.main
