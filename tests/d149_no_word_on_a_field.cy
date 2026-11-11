// trace: D149

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

struct holder:
    inner: reading;
end struct holder

fn.main [none] returns void:
    let $h: holder set_to holder[inner set_to reading[celsius set_to 1]];
    let $w: wrapped set_to $h.inner as wrapped;
    return void;
end fn.main
