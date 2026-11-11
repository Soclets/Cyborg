// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] returns void:
    let $a: reading set_to reading[celsius set_to 1];
    let $v: view<`text`: $b> set_to view[words set_to Lend $a];
    return void;
end fn.main
