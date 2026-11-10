// trace: D129, D75

fn my::finish [none] -> void:
    return void;
end fn my::finish

fn.main [none] -> void:
    finish();
    return void;
end fn.main
