// trace: D149
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

struct holder:
    inner: reading;
end struct holder

fn.main [none] -> void:
    let $h: holder = holder[inner = reading[celsius = 1]];
    let $w: wrapped = Copy $h.inner as wrapped;
    std::io::fmt_print("{}{newln}", $h.inner.celsius);
    return void;
end fn.main
