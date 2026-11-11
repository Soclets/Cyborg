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

fn warm [$r: Lent.mut reading] returns void:
    change $r.celsius set_to $r.celsius + 1;
    return void;
end fn warm

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 21, label set_to "a", at set_to 1 time.seconds, deep set_to inner[n set_to 1]];
    change $r.celsius set_to 22;
    warm(Lend.mut $r);
    change $r.label set_to "b";
    let $s: str set_to "c";
    change $r.label set_to Give $s;
    change $r.at set_to $r.at + 5 time.seconds;
    change $r.deep.n set_to 9;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $r.celsius, $r.label, $r.at, $r.deep.n);
    change $r.deep set_to inner[n set_to 10];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.deep.n);
    return void;
end fn.main
