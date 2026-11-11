// A bare `os::stdout()` is refused, asking which kind.
// trace: D273
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout();
    return void;
end fn.main
