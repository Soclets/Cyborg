// trace: D195, D51
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print.stdout("hi{newln}");
    return void;
end fn.main
