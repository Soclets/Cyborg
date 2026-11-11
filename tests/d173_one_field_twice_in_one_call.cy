// trace: D173, D2

struct inner:
    n: i32;
end struct inner

struct pair:
    a: inner;
    b: inner;
end struct pair

fn adjust [$x: Lent.mut inner, $y: Lent.mut inner] returns void:
    return void;
end fn adjust

fn.main [none] returns void:
    let mut $p: pair = pair[a = inner[n = 1], b = inner[n = 2]];
    adjust(Lend.mut $p.a, Lend.mut $p.a);
    return void;
end fn.main
