// trace: D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> void:
    return void;
end fn eat

fn pass [$b: Lent box] -> void:
    eat(Give $b);
    return void;
end fn pass

fn.main [none] -> void:
    return void;
end fn.main
