// The borrow's type names `$o`, the whole value it came from.
// trace: D172

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn.main [none] returns void:
    let $o: outer = outer[a = inner[n = 1]];
    let $l: Lend from $o inner = Lend $o.a;
    return void;
end fn.main
