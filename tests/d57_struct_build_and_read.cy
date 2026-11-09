// trace: D57
import std;
@config(default);

struct reading:
    celsius: i32;
    label: str;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[celsius = 21, label = "kitchen"];
    std::println(Lend $r.label, ": ", $r.celsius as str);
    return void;
end fn.main
