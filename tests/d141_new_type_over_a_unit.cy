// A literal still has to fit what's underneath, unit and all.
// trace: D141, D101

fam.unit time:
    time.seconds: base;
end fam.unit time

type.new timeout set_to i32[unit: time.seconds];

fn.main [none] returns void:
    let $good: timeout set_to 5 time.seconds;
    let $bad: timeout set_to 5;
    return void;
end fn.main
