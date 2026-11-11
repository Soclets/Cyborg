// A struct defines `==` and `<` for itself, and the other comparisons follow from them.
// trace: D236, D237
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

op below [$a: reading, $b: reading] returns bool:
    return $a.celsius below $b.celsius;
end op below

op == [$a: reading, $b: reading] returns bool:
    return $a.celsius == $b.celsius;
end op ==

fn shown [$yes: bool] returns str:
    if $yes:
        return "yes";
    end if
    return "no";
end fn shown

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $cold: reading set_to reading[celsius set_to 3];
    let $warm: reading set_to reading[celsius set_to 21];
    std::io::fmt_print(Lend.mut $out, "< {}{newln}", shown($cold below $warm));
    std::io::fmt_print(Lend.mut $out, "> {}{newln}", shown($cold above $warm));
    std::io::fmt_print(Lend.mut $out, "== {}{newln}", shown($cold == $warm));
    std::io::fmt_print(Lend.mut $out, "!= {}{newln}", shown($cold != $warm));
    std::io::fmt_print(Lend.mut $out, "at_least {}{newln}", shown($cold at_least $warm));
    std::io::fmt_print(Lend.mut $out, "at_most {}{newln}", shown($cold at_most $warm));
    std::io::fmt_print(Lend.mut $out, "chain {}{newln}", shown($cold below $warm below reading[celsius set_to 30]));
    return void;
end fn.main
