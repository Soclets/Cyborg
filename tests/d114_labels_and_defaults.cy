// trace: D114
import std;

fn first_word [$text: str, $limit: i32] -> i32:
    labels [$limit as limit];
    defaults [$limit = 40];
    requires $limit at_least 1;

    return $limit;
end fn first_word

fn.main [] -> void:
    let $text: str = "x";
    std::println(first_word(Lend $text, limit: 20) as str);
    std::println(first_word(Lend $text) as str);
    return void;
end fn.main
