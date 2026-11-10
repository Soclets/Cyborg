// trace: D149
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [none] -> void:
    let mut $r: reading = reading[celsius = 1];
    let $w: wrapped = Copy $r as wrapped;
    change $r.celsius = 9;
    let $back: reading = Give $w as reading;
    std::io::fmt_print("{}{newln}", $back.celsius);
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
