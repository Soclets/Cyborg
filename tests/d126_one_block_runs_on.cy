// trace: D126, D11

fn sign [$n: i32] returns i32:
    if $n above 0:
        return 1;
    else:
        let $m: i32 = $n;
    end if
end fn sign

fn.main [none] returns void:
    return void;
end fn.main
