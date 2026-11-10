// trace: D171
import std;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    std::io::fmt_print("{}{newln}", view[words = Lend $a].words.celsius);
    return void;
end fn.main
