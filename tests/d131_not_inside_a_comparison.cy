// trace: D131
@config(default);

fn.main [] -> void:
    let $b: bool = true;
    let $c: bool = true == not $b;
    return void;
end fn.main
