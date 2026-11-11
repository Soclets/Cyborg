// trace: D100

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit storage:
    storage.bytes: base;
end fam.unit storage

fn.main [none] returns void:
    let $a: i32[unit: time.seconds] set_to 1 time.seconds;
    let $b: i32[unit: time.seconds] set_to $a + 5 storage.bytes;
    return void;
end fn.main
