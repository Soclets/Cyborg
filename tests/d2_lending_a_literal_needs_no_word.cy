// trace: D2
import std;
import os;

fn f [$s: str] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn f

fn g [$s: Given str] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn g

fn.main [none] -> void:
    f("fresh");
    g("fresh too");
    return void;
end fn.main
