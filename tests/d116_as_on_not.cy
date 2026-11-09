// trace: D116
@config(default);

fn.main [] -> void:
    let $b: bool = 1 < 2;
    let $a: str = not $b as str;
    return void;
end fn.main
