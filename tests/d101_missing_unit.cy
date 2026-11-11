// trace: D101

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $b: i32[unit: time.seconds] set_to 3600;
    return void;
end fn.main
