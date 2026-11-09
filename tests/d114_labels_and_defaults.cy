// trace: D114
@config(default);
import std;

fn first_word [$text: str, $limit: i32] -> i32:
    labels [$limit as limit];
    defaults [$limit = 40];
    requires $limit at_least 1;

    return $limit;
end fn first_word

fn.main [] -> void:
    let $text: str = "x";
    std::io::fmt_print("{}{newln}", first_word(Lend $text, limit: 20));
    std::io::fmt_print("{}{newln}", first_word(Lend $text));
    return void;
end fn.main
