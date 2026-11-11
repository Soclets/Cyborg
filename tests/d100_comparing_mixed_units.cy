// trace: D100

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit storage:
    storage.bytes: base;
end fam.unit storage

fn.main [none] returns void:
    let $a: i32[unit: time.seconds] = 1 time.seconds;
    let $b: i32[unit: storage.bytes] = 1 storage.bytes;
    if $a == $b:
        return void;
    end if
    return void;
end fn.main
