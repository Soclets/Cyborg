// The borrow's type names `$o`, the whole value it came from.
// trace: D172

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn.main [none] returns void:
    let $o: outer set_to outer[a set_to inner[n set_to 1]];
    let $l: Lend from $o inner set_to Lend $o.a;
    return void;
end fn.main
