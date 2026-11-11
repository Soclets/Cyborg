// trace: D139
import std;
import os;

fn word [$bad: bool] returns str fails [too_long]:
    if $bad:
        fail too_long;
    end if
    return "fine";
end fn word

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: str set_to word(false) otherwise:
        case too_long:
            instead "…";
    end otherwise
    let $b: str set_to word(true) otherwise:
        case too_long:
            instead "…";
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
