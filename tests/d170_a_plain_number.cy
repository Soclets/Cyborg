// trace: D170, D123
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn degrees [$a: reading] -> Lend from $a i32:
    return Lend $a.celsius;
end fn degrees

fn.main [] -> void:
    return void;
end fn.main
