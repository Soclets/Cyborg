// trace: D2

struct reading:
    celsius: i32;
end struct reading

fn eat [$r: Given reading] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $r: reading = reading[celsius = 21];
    let $b: Lend from $r reading = Lend $r;
    eat(Give $b);
    return void;
end fn.main
