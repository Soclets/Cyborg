// trace: D149
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

struct holder:
    inner: wrapped;
end struct holder

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    let $h: holder = holder[inner = Give $a as wrapped];
    std::io::fmt_print("ok{newln}");
    return void;
end fn.main
