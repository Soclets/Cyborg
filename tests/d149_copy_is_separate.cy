// trace: D149
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 1];
    let $w: wrapped = Copy $r as wrapped;
    change $r.celsius = 9;
    let $back: reading = Give $w as reading;
    std::println($back.celsius as str);
    std::println($r.celsius as str);
    return void;
end fn.main
