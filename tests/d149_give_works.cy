// trace: D149
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [] -> void:
    let $r: reading = reading[celsius = 7];
    let $w: wrapped = Give $r as wrapped;
    let $back: reading = Give $w as reading;
    std::println($back.celsius as str);
    return void;
end fn.main
