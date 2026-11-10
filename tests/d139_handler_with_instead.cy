// A handler is used as if written there, and `instead` isn't a jump.
// trace: D139, D46
import std;

fam.handler word_failed:
    word_failed.use_placeholder:
        case too_long:
            instead "…";
        case no_words $n:
            std::io::fmt_print("words: {}{newln}", $n.count);
            instead "(none)";
    end word_failed.use_placeholder
end fam.handler word_failed

fn first_word [$which: i32] -> str fails [too_long, no_words[count: i32]]:
    if $which == 1:
        fail too_long;
    end if
    if $which == 2:
        fail no_words[count = 0];
    end if
    return "word";
end fn first_word

fn.main [none] -> void:
    let $a: str = first_word(1) otherwise word_failed.use_placeholder;
    let $b: str = first_word(2) otherwise word_failed.use_placeholder;
    let $c: str = first_word(3) otherwise word_failed.use_placeholder;
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $c);
    return void;
end fn.main
