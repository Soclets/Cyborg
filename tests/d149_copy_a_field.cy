// trace: D149
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

struct holder:
    inner: reading;
end struct holder

fn.main [] -> void:
    let $h: holder = holder[inner = reading[celsius = 1]];
    let $w: wrapped = Copy $h.inner as wrapped;
    std::println($h.inner.celsius as str);
    return void;
end fn.main
