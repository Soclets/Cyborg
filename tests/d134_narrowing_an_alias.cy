// trace: D134

fam.unit time:
    time.seconds: base;
end fam.unit time

type.alias score = i32;

fn.main [none] returns void:
    let $a: score.time.seconds = 1 time.seconds;
    return void;
end fn.main
