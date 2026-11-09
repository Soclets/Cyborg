// trace: D149, D3
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [] -> void:
    let $r: reading = reading[celsius = 1];
    let $w: wrapped = Copy $r as wrapped;
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
