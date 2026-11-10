// 5.8's `pair`, whose type parameters need nothing, so each says `any`.
// trace: D234, D239
@config(default);
import std;

struct pair<?left: any, ?right: any>:
    first: ?left;
    second: ?right;
end struct pair

fn first_of<?left: any, ?right: any> [$p: Given pair<?left, ?right>] -> ?left:
    return Give $p.first;
end fn first_of

fn.main [] -> void:
    let $p: pair<i32, str> = pair[first = 7, second = "seven"];
    let $q: pair<str, f64> = pair[first = "half", second = 0.5];
    std::io::fmt_print("{} {} {} {}{newln}", $p.first, $p.second, $q.first, $q.second);
    return void;
end fn.main
