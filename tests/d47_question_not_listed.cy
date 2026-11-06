// trace: D47
fn inner [] -> i32:
    fails [odd];

    return 1;
end fn inner

fn outer [] -> i32:
    fails [other];

    return inner()?;
end fn outer

fn.main [] -> void:
    return void;
end fn.main
