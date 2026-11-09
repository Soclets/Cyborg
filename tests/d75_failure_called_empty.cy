// A failure's name can clash too: the list stays bare, and every use says `my::`.
// trace: D47, D75
import std;
@config(default);

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
            std::println("too long");
        case my::empty:
            std::println("(none)");
    end otherwise
    return void;
end fn.main
