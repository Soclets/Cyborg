// trace: D171, D169
import std;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] -> void:
    let mut $a: reading = reading[celsius = 1];
    let $v: view<`text`: $a> = view[words = Lend $a];
    change $a.celsius = 5;
    std::io::fmt_print("{}{newln}", $v.words.celsius);
    return void;
end fn.main
