// trace: D171, D169
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
    change $a.celsius = 5;
    std::println($v.words.celsius as str);
    return void;
end fn.main
