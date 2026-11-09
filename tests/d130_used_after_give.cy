// trace: D130, D2
@config(default);

struct box:
    n: i32;
end struct box

fn box.%read [$b: box] -> i32:
    return $b.n;
end fn box.%read

fn eat [$b: Given box] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $b: box = box[n = 1];
    eat(Give $b);
    let $n: i32 = $b.%read();
    return void;
end fn.main
