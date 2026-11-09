// trace: D195, D51
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print.stdout("hi{newln}");
    return void;
end fn.main
