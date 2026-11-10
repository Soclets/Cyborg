// trace: D86

struct box:
    n: i32;
end struct box

fn.pure f [$b: Lent.mut box] -> i32:
    return 1;
end fn.pure f

fn.main [none] -> void:
    return void;
end fn.main
