// trace: D139, D46

fam.handler word_failed:
    word_failed.use_placeholder:
        case too_long:
            instead "…";
    end word_failed.use_placeholder
end fam.handler word_failed

fn first_word [none] returns str fails [too_long]:
    return "word";
end fn first_word

fn.main [none] returns void:
    let $a: str set_to first_word() otherwise word_failed.*use_placeholder;
    return void;
end fn.main
