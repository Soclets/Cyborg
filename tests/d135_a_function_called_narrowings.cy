// `narrowings` isn't a word of Cyborg now, so it's free as a name.
// trace: D135, D75
import std;
@config(default);

fn narrowings [] -> i32:
    return 1;
end fn narrowings

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", narrowings());
    return void;
end fn.main
