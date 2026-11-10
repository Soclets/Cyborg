// trace: D57
import std;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
    b: inner;
end struct outer

fn read [$i: inner] -> i32:
    return $i.n;
end fn read

fn.main [none] -> void:
    let $first: inner = inner[n = 1];
    let $o: outer = outer[a = Give $first, b = inner[n = 2]];
    std::io::fmt_print("{}{newln}", $o.a.n);
    std::io::fmt_print("{}{newln}", read(Lend $o.b));
    return void;
end fn.main
