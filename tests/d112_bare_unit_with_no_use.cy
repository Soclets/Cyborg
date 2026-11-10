// trace: D112

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $a: i32[unit: time.seconds] = 3600 seconds;
    return void;
end fn.main
