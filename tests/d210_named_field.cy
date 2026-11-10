// trace: D210, D247
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[celsius = 3];
    std::io::fmt_print("{$r.celsius}{newln}");
    return void;
end fn.main
