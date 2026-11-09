// `loop` and `continue` aren't words of Cyborg, so they're free as names.
// trace: D128, D75
import std;
@config(default);

fn loop [] -> void:
    std::println("called");
    return void;
end fn loop

fn.main [] -> void:
    loop();
    return void;
end fn.main
