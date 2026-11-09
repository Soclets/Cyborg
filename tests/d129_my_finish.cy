// trace: D129, D75
import std;
@config(default);

fn finish [] -> void:
    std::io::fmt_print("mine{newln}");
    return void;
end fn finish

fn.main [] -> void:
    my::finish();
    return void;
end fn.main
