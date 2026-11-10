// trace: D47

fn inner [none] -> i32 fails [odd]:
    return 1;
end fn inner

fn.main [none] -> void:
    let $n: i32 = inner()?;
    return void;
end fn.main
