// trace: D101

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $a: i32[unit: time.seconds] set_to 1 time.seconds;
    let $b: i32[unit: time.seconds] set_to $a + 5;
    return void;
end fn.main
