// trace: D134
@config(default);

type.alias a = b;
type.alias b = a;

fn.main [] -> void:
    return void;
end fn.main
