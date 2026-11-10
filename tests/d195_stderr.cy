// trace: D195
import std;

fn.main [none] -> void:
    std::io::fmt_print("out{newln}");
    std::io::fmt_print.stderr("failed{newln}");
    std::io::fmt_print("out again{newln}");
    return void;
end fn.main
