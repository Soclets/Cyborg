// trace: D149
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn look [$r: reading] -> void:
    let $w: wrapped = Copy $r as wrapped;
    std::println($r.celsius as str);
    return void;
end fn look

fn.main [] -> void:
    let $a: reading = reading[celsius = 5];
    look(Lend $a);
    return void;
end fn.main
