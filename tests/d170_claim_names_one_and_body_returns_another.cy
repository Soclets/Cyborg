// trace: D170
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading, $b: reading] -> Lend from $a reading:
    return Lend $b;
end fn first

fn.main [] -> void:
    return void;
end fn.main
