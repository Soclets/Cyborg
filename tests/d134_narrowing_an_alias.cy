// trace: D134

fam.unit time:
    time.seconds: base;
end fam.unit time

type.alias score set_to i32;

fn.main [none] returns void:
    let $a: score.time.seconds set_to 1 time.seconds;
    return void;
end fn.main
