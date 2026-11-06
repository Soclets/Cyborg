// trace: D125
import std;

narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

struct inner:
    n: i32;
end struct inner

struct reading:
    celsius: i32[min: -50, max: 50];
    label: str;
    at: i32.time.seconds;
    deep: inner;
end struct reading

fn warm [$r: Lent.mut reading] -> void:
    change $r.celsius = $r.celsius + 1;
    return void;
end fn warm

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, label = "a", at = 1 time.seconds, deep = inner[n = 1]];
    change $r.celsius = 22;
    warm(Lend.mut $r);
    change $r.label = "b";
    let $s: str = "c";
    change $r.label = Give $s;
    change $r.at = $r.at + 5 time.seconds;
    change $r.deep.n = 9;
    std::println($r.celsius as str, " ", Lend $r.label, " ", $r.at as str, " ", $r.deep.n as str);
    change $r.deep = inner[n = 10];
    std::println($r.deep.n as str);
    return void;
end fn.main
