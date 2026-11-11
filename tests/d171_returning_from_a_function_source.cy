// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn words <`t`> [$v: view<`text`: `t`>] returns Lend from `t` reading:
    return Lend $v.words;
end fn words

fn.main [none] returns void:
    return void;
end fn.main
