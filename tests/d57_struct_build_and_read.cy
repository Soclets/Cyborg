// trace: D57
import std;

struct reading:
    celsius: i32;
    label: str;
end struct reading

fn.main [none] -> void:
    let $r: reading = reading[celsius = 21, label = "kitchen"];
    std::io::fmt_print("{}: {}{newln}", $r.label, $r.celsius);
    return void;
end fn.main
