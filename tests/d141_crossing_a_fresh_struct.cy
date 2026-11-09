// A fresh value has no holder, so nothing's left behind.
// trace: D141, D2
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn make [] -> wrapped:
    return reading[celsius = 4] as wrapped;
end fn make

fn.main [] -> void:
    let $r: reading = make() as reading;
    std::println($r.celsius as str);
    return void;
end fn.main
