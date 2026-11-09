// trace: D2
@config(default);

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> void:
    return void;
end fn eat

fn pass [$b: box] -> void:
    eat(Give $b);
    return void;
end fn pass

fn.main [] -> void:
    return void;
end fn.main
