// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn pick [$a: reading] -> Lend from $a reading:
    return $a;
end fn pick

fn.main [none] -> void:
    return void;
end fn.main
