// trace: D173, D125, D2
@config(default);

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
    b: inner;
end struct outer

fn.main [] -> void:
    let mut $o: outer = outer[a = inner[n = 1], b = inner[n = 2]];
    let $l: Lend from $o inner = Lend $o.a;
    change $o.b.n = 3;
    let $n: i32 = $l.n;
    return void;
end fn.main
