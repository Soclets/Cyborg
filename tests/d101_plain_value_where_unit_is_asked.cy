// trace: D101, D100
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $x: i32 = 3600;
    let $b: i32.time.seconds = $x;
    return void;
end fn.main
