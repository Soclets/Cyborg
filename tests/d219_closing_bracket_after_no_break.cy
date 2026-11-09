// A closing bracket goes on its own line only when its opening one broke the line.
// trace: D219
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{} {}{newln}", 1,
        2
    );
    return void;
end fn.main
