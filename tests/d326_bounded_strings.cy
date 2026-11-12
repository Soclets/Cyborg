// A string with a finite `len.max` holds its bytes inside the value, and lending it gives a
// view, which a `str` parameter takes.
// trace: D326, D327
import std;
import os;

fn shout [$out: Lent.mut os::stdout.unbuffered, $text: str] -> void:
    std::io::fmt_print(Lend.mut $out, "{}!{newln}", $text);
    return void;
end fn shout

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $name: str[len.max: 6 bytes] = "Tankun";
    let $nothing: str[len.max: 6 bytes] = empty;
    shout(Lend.mut $out, Lend $name);
    shout(Lend.mut $out, "hi");
    std::io::fmt_print(Lend.mut $out, "[{$nothing}]{newln}");
    return void;
end fn.main
