// trace: D161
import std;
import os;

fn show [$x: f64.stable] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn show

fn.main [none] -> void:
    show(0.5);
    show(1.0);
    show(-1.5);
    show(100.0);
    show(~0.3);
    show(~3.14159265358979323846);
    show(~1.7976931348623157);
    return void;
end fn.main
