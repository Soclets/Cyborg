// trace: D9
@config(default);

fn.main [] -> void:
    let $b: bool = 1 < 2;
    if $b: bool:
        return void;
    end if
    return void;
end fn.main
