// trace: D101, D100

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $x: i32 = 3600;
    let $b: i32[unit: time.seconds] = $x;
    return void;
end fn.main
