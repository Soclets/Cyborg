// Every type parameter says what it needs, `any` when it needs nothing.
// trace: D239
@config(default);

struct pair<?left, ?right: any>:
    first: ?left;
    second: ?right;
end struct pair

fn.main [] -> void:
    return void;
end fn.main
