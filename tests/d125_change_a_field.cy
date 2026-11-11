// trace: D125
import std;
import os;

fam.unit time:
    time.seconds: base;
end fam.unit time

struct inner:
    n: i32;
end struct inner

struct reading:
    celsius: i32[min: -50, max: 50];
    label: str;
    at: i32[unit: time.seconds];
    deep: inner;
end struct reading

fn warm [$r: Lent.mut reading] -> void:
    change $r.celsius = $r.celsius + 1;
    return void;
end fn warm

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21, label = "a", at = 1 time.seconds, deep = inner[n = 1]];
    change $r.celsius = 22;
    warm(Lend.mut $r);
    change $r.label = "b";
    let $s: str = "c";
    change $r.label = Give $s;
    change $r.at = $r.at + 5 time.seconds;
    change $r.deep.n = 9;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $r.celsius, $r.label, $r.at, $r.deep.n);
    change $r.deep = inner[n = 10];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.deep.n);
    return void;
end fn.main
