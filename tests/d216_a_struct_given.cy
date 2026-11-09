// trace: D216, D3
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[celsius = 1];
    let $s := Give $r;
    let $t := reading[celsius = 2];
    std::io::fmt_print("{} {}{newln}", $s.celsius, $t.celsius);
    return void;
end fn.main
