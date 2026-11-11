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
    let $a: reading = reading[celsius = 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", view[words = Lend $a].words.celsius);
    return void;
end fn.main
