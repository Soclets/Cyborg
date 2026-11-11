// trace: D171

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $a: reading set_to reading[celsius set_to 1];
    let $r: reading<`text`: $a> set_to reading[celsius set_to 2];
    return void;
end fn.main
