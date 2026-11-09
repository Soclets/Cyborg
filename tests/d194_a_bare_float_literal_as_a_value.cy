// The standard's fourth example prints `12.5` with nothing saying which float it is.
// trace: D194, D59
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{form: science, places: 2}{newln}", 12.5);
    return void;
end fn.main
