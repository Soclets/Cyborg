// trace: D124

fn.main [none] returns void:
    let $n: i32 set_to 0;
    if $n != 0 and.both 10 div.trunc $n above 1:
        return void;
    end if
    return void;
end fn.main
