// trace: D171
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [] -> void:
    let mut $a: reading = reading[celsius = 1];
    let $v: view<`text`: $a> = view[words = Lend $a];
    std::io::fmt_print("{}{newln}", $a.celsius);
    std::io::fmt_print("{}{newln}", $v.words.celsius);
    change $a.celsius = 5;
    std::io::fmt_print("{}{newln}", $a.celsius);
    return void;
end fn.main
