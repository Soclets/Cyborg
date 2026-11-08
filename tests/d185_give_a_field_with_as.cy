// A field moves out of what you own, and `as` crosses it like any whole value.
// trace: D185, D149
struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

struct holder:
    inner: reading;
end struct holder

fn.main [] -> void:
    let $h: holder = holder[inner = reading[celsius = 1]];
    let $w: wrapped = Give $h.inner as wrapped;
    return void;
end fn.main
