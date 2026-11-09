// trace: D126, D33
@config(default);

fn.main [] -> void:
    if 1 < 2:
        let $a: i32 = 1;
    else:
        let $b: i32 = $a;
    end if
    return void;
end fn.main
