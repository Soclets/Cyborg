// trace: D3

struct box:
    n: i32;
end struct box

fn keep [$b: Given box] returns box:
    return $b;
end fn keep

fn.main [none] returns void:
    return void;
end fn.main
