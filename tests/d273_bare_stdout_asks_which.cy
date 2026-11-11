// A bare `os::stdout()` is refused, asking which kind.
// trace: D273
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout();
    return void;
end fn.main
