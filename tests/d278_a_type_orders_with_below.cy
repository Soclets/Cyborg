// A type writes `==` and `op below`, and `above`, `at_least`, `at_most` and `!=` follow.
// trace: D278, D237
import std;
import os;

struct money:
    cents: i64;
end struct money

op == [$a: money, $b: money] returns bool:
    return $a.cents == $b.cents;
end op ==

op below [$a: money, $b: money] returns bool:
    return $a.cents below $b.cents;
end op below

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $small: money set_to money[cents set_to 5];
    let $big: money set_to money[cents set_to 9];
    if $small below $big:
        std::io::fmt_print(Lend.mut $out, "below{newln}");
    end if
    if $small above $big:
        std::io::fmt_print(Lend.mut $out, "above{newln}");
    end if
    if $big at_least $small:
        std::io::fmt_print(Lend.mut $out, "at_least{newln}");
    end if
    if $big at_most $small:
        std::io::fmt_print(Lend.mut $out, "at_most{newln}");
    end if
    if $small != $big:
        std::io::fmt_print(Lend.mut $out, "!={newln}");
    end if
    return void;
end fn.main
