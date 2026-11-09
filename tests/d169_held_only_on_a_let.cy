// trace: D169, D170
@config(default);

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn first [$a: reading] -> Lend.held from $a reading:
    return Lend $a;
end fn first

fn.main [] -> void:
    return void;
end fn.main
