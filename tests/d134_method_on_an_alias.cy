// trace: D134, D130

type.alias score = i32;

fn score.%same [$s: score] -> score:
    return $s;
end fn score.%same

fn.main [none] -> void:
    return void;
end fn.main
