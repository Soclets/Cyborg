// trace: D129, D75
@config(default);
import std;

fn my::finish [] -> void:
    std::io::fmt_print("mine{newln}");
    return void;
end fn my::finish

fn.main [] -> void:
    my::finish();
    return void;
end fn.main
