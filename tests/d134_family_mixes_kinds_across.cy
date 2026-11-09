// trace: D134
@config(default);

fam.type.new ids:
    ids.user = i64;
end fam.type.new ids

fam.type.alias ids:
    ids.order = i64;
end fam.type.alias ids

fn.main [] -> void:
    return void;
end fn.main
