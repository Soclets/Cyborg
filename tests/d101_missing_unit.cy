// trace: D101

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $b: i32[unit: time.seconds] = 3600;
    return void;
end fn.main
