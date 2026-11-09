// trace: D47, D75
@config(default);

fn first_word [] -> void:
    fails [empty];

    fail empty;
end fn first_word

fn.main [] -> void:
    return void;
end fn.main
