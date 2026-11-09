// trace: D57
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    if 1 < 2:
        let $a: i32 = 2;
    end if
    return void;
end fn.main
