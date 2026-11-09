// trace: D3
@config(default);

struct box:
    n: i32;
end struct box

fn keep [$b: Given box] -> box:
    return $b;
end fn keep

fn.main [] -> void:
    return void;
end fn.main
