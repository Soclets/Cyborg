// trace: D194
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $r: reading = reading[celsius = 1];
    std::io::fmt_print("{}{newln}", $r);
    return void;
end fn.main
