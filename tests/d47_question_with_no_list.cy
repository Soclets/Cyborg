// `?` in a function that lists no failures isn't decided.
// trace: D47
@config(default);

fn inner [] -> i32 fails [odd]:
    return 1;
end fn inner

fn outer [] -> i32:
    return inner()?;
end fn outer

fn.main [] -> void:
    return void;
end fn.main
