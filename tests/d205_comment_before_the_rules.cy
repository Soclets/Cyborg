// trace: D205, D204
// Comments can go anywhere, even above the rules line.
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("fine{newln}");
    return void;
end fn.main
