// trace: D210, D247
@config(default);
import std;

fn.main [] -> void:
    let $t: f64.stable = 2.5;
    std::io::fmt_print("{$t, places: 2}{newln}");
    return void;
end fn.main
