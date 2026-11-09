// trace: D2
import std;
@config(default);

fn f [$s: str] -> void:
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn f

fn g [$s: Given str] -> void:
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn g

fn.main [] -> void:
    f("fresh");
    g("fresh too");
    return void;
end fn.main
