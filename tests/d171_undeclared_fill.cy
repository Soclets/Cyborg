// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] returns void:
    let $a: reading = reading[celsius = 1];
    let $v: view<`text`: $b> = view[words = Lend $a];
    return void;
end fn.main
