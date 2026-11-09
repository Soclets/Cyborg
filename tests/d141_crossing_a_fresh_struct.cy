// A fresh value has no holder, so nothing's left behind.
// trace: D141, D2
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn make [] -> wrapped:
    return reading[celsius = 4] as wrapped;
end fn make

fn.main [] -> void:
    let $r: reading = make() as reading;
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
