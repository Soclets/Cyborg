// trace: D127
import std;
import os;

struct flags:
    on: bool;
end struct flags

fn yes_or_no [$b: bool] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if $b:
        std::io::fmt_print(Lend.mut $out, "yes{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "no{newln}");
    end if
    return void;
end fn yes_or_no

fn pick [$b: bool] returns bool:
    labels [$b as flag];
    defaults [$b = true];

    return $b;
end fn pick

const $always: bool = true;

fn.main [none] returns void:
    let $a: bool = true;
    let mut $b: bool = false;
    yes_or_no($a);
    yes_or_no($b);
    change $b = not $b;
    yes_or_no($b);
    yes_or_no(true and false);
    yes_or_no(true or false);
    yes_or_no(true == false);
    yes_or_no($always != false);
    yes_or_no(pick());
    yes_or_no(pick(flag: false));
    let $f: flags = flags[on = true];
    yes_or_no($f.on);
    let $all: std::array<bool>[len: 2 items] = all true;
    for $x in $all:
        yes_or_no($x);
    end for
    return void;
end fn.main
