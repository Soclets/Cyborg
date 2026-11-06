// `narrowings` isn't a word of Cyborg now, so it's free as a name.
// trace: D135, D75
import std;

fn narrowings [] -> i32:
    return 1;
end fn narrowings

fn.main [] -> void:
    std::println(narrowings() as str);
    return void;
end fn.main
