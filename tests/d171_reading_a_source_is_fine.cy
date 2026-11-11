// trace: D171
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $a: reading = reading[celsius = 1];
    let $v: view<`text`: $a> = view[words = Lend $a];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.celsius);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v.words.celsius);
    change $a.celsius = 5;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.celsius);
    return void;
end fn.main
