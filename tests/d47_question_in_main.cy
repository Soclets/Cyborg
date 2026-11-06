// trace: D47
fn inner [] -> i32:
    fails [odd];

    return 1;
end fn inner

fn.main [] -> void:
    let $n: i32 = inner()?;
    return void;
end fn.main
