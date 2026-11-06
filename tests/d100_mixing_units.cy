// trace: D100
narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

narrowings.unit storage:
    storage.bytes: base;
end narrowings.unit storage

fn.main [] -> void:
    let $a: i32.time.seconds = 1 time.seconds;
    let $b: i32.time.seconds = $a + 5 storage.bytes;
    return void;
end fn.main
