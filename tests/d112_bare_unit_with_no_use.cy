// trace: D112
narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

fn.main [] -> void:
    let $a: i32.time.seconds = 3600 seconds;
    return void;
end fn.main
