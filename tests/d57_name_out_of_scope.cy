// trace: D57, D33
@config(default);

fn.main [] -> void:
    if 1 < 2:
        let $a: i32 = 1;
    end if
    let $b: i32 = $a;
    return void;
end fn.main
