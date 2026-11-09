// trace: D141, D20
@config(default);

type.new percent = i32[min: 0, max: 100];

fn.main [] -> void:
    let $p: percent = 150;
    return void;
end fn.main
