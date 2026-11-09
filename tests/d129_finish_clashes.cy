// trace: D129, D75
@config(default);

fn finish [] -> void:
    return void;
end fn finish

fn.main [] -> void:
    finish();
    return void;
end fn.main
