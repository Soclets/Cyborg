// trace: D171
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [] -> void:
    let mut $a: reading = reading[celsius = 1];
    let $v: view<`text`: $a> = view[words = Lend $a];
    std::println($a.celsius as str);
    std::println($v.words.celsius as str);
    change $a.celsius = 5;
    std::println($a.celsius as str);
    return void;
end fn.main
