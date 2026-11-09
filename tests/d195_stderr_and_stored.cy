// trace: D195
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print.stored.stderr("{}{newln}", 0.5);
    return void;
end fn.main
