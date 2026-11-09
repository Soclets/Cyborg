// What unit comes back isn't decided.
// trace: D100
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: i32.time.seconds = 1 time.seconds;
    let $b: i32.time.seconds = $a x 2;
    return void;
end fn.main
