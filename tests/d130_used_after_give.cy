// trace: D130, D2

struct box:
    n: i32;
end struct box

fn box.%read [$b: box] returns i32:
    return $b.n;
end fn box.%read

fn eat [$b: Given box] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    eat(Give $b);
    let $n: i32 set_to $b.%read();
    return void;
end fn.main
