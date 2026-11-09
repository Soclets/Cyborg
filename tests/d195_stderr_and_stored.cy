// They combine, but their order isn't settled, so either order is NotDesignedYet.
// trace: D195, D221
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print.stored.stderr("{}{newln}", 0.5);
    return void;
end fn.main
