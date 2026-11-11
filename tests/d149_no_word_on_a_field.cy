// trace: D149

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

struct holder:
    inner: reading;
end struct holder

fn.main [none] returns void:
    let $h: holder = holder[inner = reading[celsius = 1]];
    let $w: wrapped = $h.inner as wrapped;
    return void;
end fn.main
