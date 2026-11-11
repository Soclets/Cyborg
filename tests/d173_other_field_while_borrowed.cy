// trace: D173, D125, D2

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
    b: inner;
end struct outer

fn.main [none] returns void:
    let mut $o: outer set_to outer[a set_to inner[n set_to 1], b set_to inner[n set_to 2]];
    let $l: Lend from $o inner set_to Lend $o.a;
    change $o.b.n set_to 3;
    let $n: i32 set_to $l.n;
    return void;
end fn.main
