// The standard's `shown`: a parameter names the borrow inside it with the function's own source.
// trace: D171
import std;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn shown <`t`> [$v: view<`text`: `t`>] -> i32:
    return $v.words.celsius;
end fn shown

fn.main [none] -> void:
    let $a: reading = reading[celsius = 9];
    let $v: view<`text`: $a> = view[words = Lend $a];
    std::io::fmt_print("{}{newln}", shown(Lend $v));
    return void;
end fn.main
