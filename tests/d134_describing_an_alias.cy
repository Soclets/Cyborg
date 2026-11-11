// trace: D134

type.alias score set_to i32;

fn.main [none] returns void:
    let $a: score[min: 0, max: 5] set_to 1;
    return void;
end fn.main
