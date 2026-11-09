// trace: D139
@config(default);
import std;

fn word [$bad: bool] -> str fails [too_long]:
    if $bad:
        fail too_long;
    end if
    return "fine";
end fn word

fn.main [] -> void:
    let $a: str = word(false) otherwise:
        case too_long:
            instead "…";
    end otherwise
    let $b: str = word(true) otherwise:
        case too_long:
            instead "…";
    end otherwise
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
