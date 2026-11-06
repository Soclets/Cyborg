// trace: D101, D100
narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

fn.main [] -> void:
    let $x: i32 = 3600;
    let $b: i32.time.seconds = $x;
    return void;
end fn.main
