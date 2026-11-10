// The first release on a day has no counter, so `.1` is refused.
// trace: D231
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
