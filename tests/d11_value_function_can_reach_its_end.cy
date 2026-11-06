// trace: D11
fn f [$n: i32] -> i32:
    if $n at_least 1:
        return 1;
    end if
end fn f

fn.main [] -> void:
    return void;
end fn.main
