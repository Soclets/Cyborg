// trace: D170
struct reading:
    celsius: i32;
end struct reading

fn pick [$a: reading] -> Lend from program reading:
    return Lend $a;
end fn pick

fn.main [] -> void:
    return void;
end fn.main
