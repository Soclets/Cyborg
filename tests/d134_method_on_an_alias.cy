// trace: D134, D130
@config(default);

type.alias score = i32;

fn score.%same [$s: score] -> score:
    return $s;
end fn score.%same

fn.main [] -> void:
    return void;
end fn.main
