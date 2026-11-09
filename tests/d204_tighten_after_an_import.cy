// trace: D204
@config(default);
import std;
@tighten_config(var::mut::never_changed = refuse);

fn.main [] -> void:
    return void;
end fn.main
