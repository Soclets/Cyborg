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
    let mut $p: pair set_to pair[a set_to inner[n set_to 1], b set_to inner[n set_to 2]];
    adjust(Lend.mut $p.a, Lend.mut $p.a);
    return void;
end fn.main
