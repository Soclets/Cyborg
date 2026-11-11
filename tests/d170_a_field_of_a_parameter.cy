// trace: D170, D172
import std;
import os;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn part [$o: outer] returns Lend from $o inner:
    return Lend $o.a;
end fn part

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $o: outer set_to outer[a set_to inner[n set_to 7]];
    let $p: Lend from $o inner set_to part(Lend $o);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $p.n);
    return void;
end fn.main
