// trace: D100
fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit storage:
    storage.bytes: base;
end fam.unit storage

fn.main [] -> void:
    let $a: i32.time.seconds = 1 time.seconds;
    let $b: i32.time.seconds = $a + 5 storage.bytes;
    return void;
end fn.main
