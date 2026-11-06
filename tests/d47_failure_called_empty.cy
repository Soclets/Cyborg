// 8.6 uses `empty` as a failure's name, bare, though it's also a keyword.
// trace: D47, D75
import std;

fn first_word [$bad: bool] -> void:
    fails [too_long, empty];

    if $bad:
        fail empty;
    end if
    return void;
end fn first_word

fn.main [] -> void:
    first_word(1 < 2) otherwise:
        case too_long:
            std::println("too long");
        case empty:
            std::println("(none)");
    end otherwise
    return void;
end fn.main
