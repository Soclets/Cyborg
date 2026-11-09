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
    let $a: reading = reading[celsius = 1];
    std::println(view[words = Lend $a].words.celsius as str);
    return void;
end fn.main
