// `$b` isn't used after the read, so the borrow is over before `Give`.
// trace: D169, D2

struct reading:
    celsius: i32;
end struct reading

fn eat [$r: Given reading] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 21];
    let $b: Lend from $r reading set_to Lend $r;
    let $n: i32 set_to $b.celsius;
    eat(Give $r);
    return void;
end fn.main
