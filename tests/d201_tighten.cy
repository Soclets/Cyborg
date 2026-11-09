// trace: D201
@config(default);
@tighten_config(var::mut::never_changed = refuse);

fn.main [] -> void:
    return void;
end fn.main
