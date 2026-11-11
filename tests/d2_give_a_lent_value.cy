// trace: D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns void:
    return void;
end fn eat

fn pass [$b: box] returns void:
    eat(Give $b);
    return void;
end fn pass

fn.main [none] returns void:
    return void;
end fn.main
