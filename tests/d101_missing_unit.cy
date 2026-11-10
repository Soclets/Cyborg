// trace: D101
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $b: i32[unit: time.seconds] = 3600;
    return void;
end fn.main
