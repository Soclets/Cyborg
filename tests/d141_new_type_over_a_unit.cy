// A literal still has to fit what's underneath, unit and all.
// trace: D141, D101
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

type.new timeout = i32.time.seconds;

fn.main [] -> void:
    let $good: timeout = 5 time.seconds;
    let $bad: timeout = 5;
    return void;
end fn.main
