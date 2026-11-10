// Named holes don't mix with `{}` or numbered ones in one format.
// trace: D247
@config(default);
import std;

fn.main [] -> void:
    let $name: str = "rope";
    std::io::fmt_print("{$name} and {}{newln}", 3);
    return void;
end fn.main
