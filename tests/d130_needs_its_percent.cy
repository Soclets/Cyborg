// trace: D130
@config(default);

struct box:
    n: i32;
end struct box

fn box.f [$b: box] -> i32:
    return $b.n;
end fn box.f

fn.main [] -> void:
    return void;
end fn.main
