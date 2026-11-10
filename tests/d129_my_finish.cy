// trace: D129, D75
import std;

fn my::finish [none] -> void:
    std::io::fmt_print("mine{newln}");
    return void;
end fn my::finish

fn.main [none] -> void:
    my::finish();
    return void;
end fn.main
