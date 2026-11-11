// A failure's name can clash too: the list stays bare, and every use says `my::`.
// trace: D47, D75
import std;
import os;

fn first_word [$bad: bool] returns void fails [too_long, empty]:
    if $bad:
        fail my::empty;
    end if
    return void;
end fn first_word

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    first_word(1 below 2) otherwise:
        case too_long:
            std::io::fmt_print(Lend.mut $out, "too long{newln}");
        case my::empty:
            std::io::fmt_print(Lend.mut $out, "(none){newln}");
    end otherwise
    return void;
end fn.main
