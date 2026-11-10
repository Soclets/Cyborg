// Named holes don't mix with `{}` or numbered ones in one format.
// trace: D247
import std;

fn.main [none] -> void:
    let $name: str = "rope";
    std::io::fmt_print("{$name} and {}{newln}", 3);
    return void;
end fn.main
