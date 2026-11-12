// A packed field could sit at an odd address, so borrowing it is refused: copy it first.
// trace: D320
struct.packed header:
    kind: u8;
    length: u32;
end struct.packed header

fn grow [$n: Lent.mut u32] -> void:
    change $n = $n + 1;
    return void;
end fn grow

fn.main [none] -> void:
    let mut $h: header = header[kind = 1, length = 512];
    grow(Lend.mut $h.length);
    return void;
end fn.main
