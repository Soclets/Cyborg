// trace: D127
import std;
import os;

struct flags:
    on: bool;
end struct flags

fn yes_or_no [$b: bool] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if $b:
        std::io::fmt_print(Lend.mut $out, "yes{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "no{newln}");
    end if
    return void;
end fn yes_or_no

fn pick [$b: bool] returns bool:
    labels [$b as flag];
    defaults [$b set_to true];

    return $b;
end fn pick

const $always: bool set_to true;

fn.main [none] returns void:
    let $a: bool set_to true;
    let mut $b: bool set_to false;
    yes_or_no($a);
    yes_or_no($b);
    change $b set_to not $b;
    yes_or_no($b);
    yes_or_no(true and false);
    yes_or_no(true or false);
    yes_or_no(true == false);
    yes_or_no($always != false);
    yes_or_no(pick());
    yes_or_no(pick(flag: false));
    let $f: flags set_to flags[on set_to true];
    yes_or_no($f.on);
    let $all: std::array<bool>[len: 2 items] set_to all true;
    for $x in $all:
        yes_or_no($x);
    end for
    return void;
end fn.main
