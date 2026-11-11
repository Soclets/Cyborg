// `=` sets a field, and `:` names a label.
// trace: D57

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $r: reading set_to reading[celsius: 21];
    return void;
end fn.main
