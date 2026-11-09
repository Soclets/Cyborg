// trace: D134, D20
@config(default);

type.alias percent = i32[min: 0, max: 100];

fn.main [] -> void:
    let $a: percent = 150;
    return void;
end fn.main
