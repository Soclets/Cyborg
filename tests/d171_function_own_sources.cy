// The standard's `shown`: a parameter names the borrow inside it with the function's own source.
// trace: D171
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn shown <`t`> [$v: view<`text`: `t`>] returns i32:
    return $v.words.celsius;
end fn shown

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: reading set_to reading[celsius set_to 9];
    let $v: view<`text`: $a> set_to view[words set_to Lend $a];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", shown(Lend $v));
    return void;
end fn.main
