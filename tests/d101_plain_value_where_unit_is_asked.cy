// trace: D101, D100

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $x: i32 set_to 3600;
    let $b: i32[unit: time.seconds] set_to $x;
    return void;
end fn.main
