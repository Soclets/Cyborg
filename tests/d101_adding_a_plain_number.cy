// trace: D101
narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

fn.main [] -> void:
    let $a: i32.time.seconds = 1 time.seconds;
    let $b: i32.time.seconds = $a + 5;
    return void;
end fn.main
