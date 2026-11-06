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

fn.main [] -> void:
    let $first: inner = inner[n = 1];
    let $o: outer = outer[a = Give $first, b = inner[n = 2]];
    std::println($o.a.n as str);
    std::println(read(Lend $o.b) as str);
    return void;
end fn.main
