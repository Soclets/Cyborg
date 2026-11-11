// trace: D170, D172
import std;
import os;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn part [$o: outer] -> Lend from $o inner:
    return Lend $o.a;
end fn part

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $o: outer = outer[a = inner[n = 7]];
    let $p: Lend from $o inner = part(Lend $o);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $p.n);
    return void;
end fn.main
