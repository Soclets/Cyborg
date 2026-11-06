// trace: D130, D125
import std;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn inner.%bump [$i: Lent.mut inner] -> void:
    change $i.n = $i.n + 1;
    return void;
end fn inner.%bump

fn.main [] -> void:
    let mut $o: outer = outer[a = inner[n = 1]];
    $o.a.%bump();
    std::println($o.a.n as str);
    return void;
end fn.main
