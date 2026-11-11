// Named holes don't mix with `{}` or numbered ones in one format.
// trace: D247
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $name: str = "rope";
    std::io::fmt_print(Lend.mut $out, "{$name} and {}{newln}", 3);
    return void;
end fn.main
