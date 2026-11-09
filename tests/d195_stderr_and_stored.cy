// trace: D195
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print.stored.stderr("{}{newln}", 0.5);
    return void;
end fn.main
