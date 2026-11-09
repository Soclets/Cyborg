// They combine, but their order isn't settled, so either order is NotDesignedYet.
// trace: D221
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print.stderr.stored("{}{newln}", 0.5);
    return void;
end fn.main
