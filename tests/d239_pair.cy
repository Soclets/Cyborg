// 5.8's `pair`, whose type parameters need nothing, so each says `any`.
// trace: D234, D239
import std;
import os;

struct pair<?left: any.type, ?right: any.type>:
    first: ?left;
    second: ?right;
end struct pair

fn first_of<?left: any.type, ?right: any.type> [$p: Given pair<?left, ?right>] returns ?left:
    return Give $p.first;
end fn first_of

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $p: pair<i32, str> set_to pair[first set_to 7, second set_to "seven"];
    let $q: pair<str, f64.stable> set_to pair[first set_to "half", second set_to 0.5];
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $p.first, $p.second, $q.first, $q.second);
    return void;
end fn.main
