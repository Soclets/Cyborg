// trace: D86
@config(default);

struct box:
    n: i32;
end struct box

fn.pure f [$b: Lent.mut box] -> i32:
    return 1;
end fn.pure f

fn.main [] -> void:
    return void;
end fn.main
