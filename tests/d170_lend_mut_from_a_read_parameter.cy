// trace: D170, D2

struct reading:
    celsius: i32;
end struct reading

fn pick [$a: reading] -> Lend.mut from $a reading:
    return Lend.mut $a;
end fn pick

fn.main [none] -> void:
    return void;
end fn.main
