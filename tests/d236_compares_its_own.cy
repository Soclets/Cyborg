// A struct defines `==` and `<` for itself, and the other comparisons follow from them.
// trace: D236, D237
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

op < [$a: reading, $b: reading] -> bool:
    return $a.celsius < $b.celsius;
end op <

op == [$a: reading, $b: reading] -> bool:
    return $a.celsius == $b.celsius;
end op ==

fn shown [$yes: bool] -> str:
    if $yes:
        return "yes";
    end if
    return "no";
end fn shown

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $cold: reading = reading[celsius = 3];
    let $warm: reading = reading[celsius = 21];
    std::io::fmt_print(Lend.mut $out, "< {}{newln}", shown($cold < $warm));
    std::io::fmt_print(Lend.mut $out, "> {}{newln}", shown($cold > $warm));
    std::io::fmt_print(Lend.mut $out, "== {}{newln}", shown($cold == $warm));
    std::io::fmt_print(Lend.mut $out, "!= {}{newln}", shown($cold != $warm));
    std::io::fmt_print(Lend.mut $out, "at_least {}{newln}", shown($cold at_least $warm));
    std::io::fmt_print(Lend.mut $out, "at_most {}{newln}", shown($cold at_most $warm));
    std::io::fmt_print(Lend.mut $out, "chain {}{newln}", shown($cold < $warm < reading[celsius = 30]));
    return void;
end fn.main
