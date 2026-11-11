// trace: D171
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: reading set_to reading[celsius set_to 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", view[words set_to Lend $a].words.celsius);
    return void;
end fn.main
