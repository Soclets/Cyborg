// trace: D47

fn inner [none] returns i32 fails [odd]:
    return 1;
end fn inner

fn outer [none] returns i32 fails [other]:
    return inner()?;
end fn outer

fn.main [none] returns void:
    return void;
end fn.main
