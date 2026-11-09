// trace: D187, D82
@config(default);
import @std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
