// trace: D114
import std;
import os;

fn first_word [$text: str, $limit: i32] returns i32:
    labels [$limit as limit];
    defaults [$limit set_to 40];
    requires $limit at_least 1;

    return $limit;
end fn first_word

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $text: str set_to "x";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first_word(Lend $text, limit: 20));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first_word(Lend $text));
    return void;
end fn.main
