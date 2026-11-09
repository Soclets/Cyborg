// trace: D124
@config(default);

fn.main [] -> void:
    let $n: i32 = 0;
    if $n != 0 and.both 10 div.trunc $n > 1:
        return void;
    end if
    return void;
end fn.main
