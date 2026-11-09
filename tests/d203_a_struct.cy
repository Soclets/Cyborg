// trace: D203, D3
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    decl $r: reading;
    assign $r = reading[celsius = 5];
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
