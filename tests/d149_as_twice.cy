// The word is on the whole value, so the second `as` is on something fresh.
// trace: D149
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

type.new rewrapped = wrapped;

fn.main [] -> void:
    let $r: reading = reading[celsius = 1];
    let $w: rewrapped = Give $r as wrapped as rewrapped;
    std::io::fmt_print("ok{newln}");
    return void;
end fn.main
