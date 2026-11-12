// A call fills type parameters from a generic struct's instance, field by field, so
// `first_of` gives back a `pair`'s first, whatever fills it.
// trace: D241, D234, D239
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

struct pair<?left: any.type, ?right: any.type>:
    first: ?left;
    second: ?right;
end struct pair

fn first_of<?left: any.type, ?right: any.type> [$p: Given pair<?left, ?right>] -> ?left:
    return Give $p.first;
end fn first_of

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $p: pair<i32, str> = pair[first = 7, second = "seven"];
    let $q: pair<reading, i64> = pair[first = reading[celsius = 21], second = 3];
    let $n: i32 = first_of(Give $p);
    let $r: reading = first_of(Give $q);
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $n, $r.celsius);
    return void;
end fn.main
