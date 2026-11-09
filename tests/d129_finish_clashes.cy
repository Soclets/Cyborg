// trace: D129, D75
@config(default);

fn my::finish [] -> void:
    return void;
end fn my::finish

fn.main [] -> void:
    finish();
    return void;
end fn.main
