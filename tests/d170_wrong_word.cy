// trace: D170
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn pick [$a: Lent.mut reading] -> Lend from $a reading:
    return Lend.mut $a;
end fn pick

fn.main [] -> void:
    return void;
end fn.main
