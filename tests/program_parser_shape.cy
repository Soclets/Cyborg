// The shape of a small tokenizer step: a union of tokens, passed around and given away.
// trace: D2, D3, D9, D33, D57, D104
import std;
import os;

union.tagged token:
    number[value: i32];
    word[text: str[len.max: 32 bytes]];
    done;
end union.tagged token

fn describe [$t: Given token] -> str[len.max: 32 bytes]:
    match $t:
        case token.number $n:
            return "(number)";
        case token.word $w:
            return Copy $w.text;
        case token.done:
            return "(done)";
    end match
end fn describe

fn make_word [$text: Given str[len.max: 32 bytes]] -> token:
    return token.word[text = Give $text];
end fn make_word

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $name: str[len.max: 32 bytes] = "hello";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", describe(token.number[value = 42]));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", describe(make_word(Give $name)));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", describe(token.done));

    let mut $state: token = token.number[value = 1];
    if $state: token.number:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $state.value);
        change $state = token.done;
    end if
    if $state: token.done:
        std::io::fmt_print(Lend.mut $out, "finished{newln}");
    end if
    return void;
end fn.main
