// trace: D170, D172
import std;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn part [$o: outer] -> Lend from $o inner:
    return Lend $o.a;
end fn part

fn.main [] -> void:
    let $o: outer = outer[a = inner[n = 7]];
    let $p: Lend from $o inner = part(Lend $o);
    std::println($p.n as str);
    return void;
end fn.main
