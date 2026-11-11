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
    let mut $a: reading set_to reading[celsius set_to 1];
    let $v: view<`text`: $a> set_to view[words set_to Lend $a];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.celsius);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v.words.celsius);
    change $a.celsius set_to 5;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.celsius);
    return void;
end fn.main
