// A failure's name can clash too: the list stays bare, and every use says `my::`.
// trace: D47, D75
@config(default);
import std;

fn first_word [$bad: bool] -> void:
    fails [too_long, empty];

    if $bad:
        fail my::empty;
    end if
    return void;
end fn first_word

fn.main [] -> void:
    first_word(1 < 2) otherwise:
        case too_long:
            std::io::fmt_print("too long{newln}");
        case my::empty:
            std::io::fmt_print("(none){newln}");
    end otherwise
    return void;
end fn.main
