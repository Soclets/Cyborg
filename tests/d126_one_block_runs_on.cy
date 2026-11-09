// trace: D126, D11
@config(default);

fn sign [$n: i32] -> i32:
    if $n > 0:
        return 1;
    else:
        let $m: i32 = $n;
    end if
end fn sign

fn.main [] -> void:
    return void;
end fn.main
