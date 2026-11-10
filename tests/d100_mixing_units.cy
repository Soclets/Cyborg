// trace: D100
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit storage:
    storage.bytes: base;
end fam.unit storage

fn.main [] -> void:
    let $a: i32[unit: time.seconds] = 1 time.seconds;
    let $b: i32[unit: time.seconds] = $a + 5 storage.bytes;
    return void;
end fn.main
