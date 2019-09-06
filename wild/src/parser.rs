
#[derive(Debug)]
enum State {
    BetweenArgs,
    InArg(bool),
    OnQuote,
    Backslashes(usize, bool),
}

/// Given UCS2/potentially-broken-UTF-16 string parses one argument, following
/// the absolutely bizarre quoting rules of `CommandLineToArgvW`, and returns
/// parsed argument as well as a slice of the remaining arguments.
///
/// Calling this repeatedly until rest is empty will parse all arguments.
///
/// `arg` is an empty pre-allocated argument to be returned, and the callback adds a new code unit to it.
/// The last callback argument is whether the unit was quoted or not.
///
/// This parses u16 code units, rather than code points.
/// This allows supporting unpaired surrogates and ensures they won't "eat" any control characters.
pub fn next_arg<AddC, ArgVec>(line: &[u16], mut arg: ArgVec, push: AddC) -> (Option<ArgVec>, &[u16])
where
    AddC: Fn(&mut ArgVec, u16, bool),
{
    use self::State::*;
    let mut state = BetweenArgs;
    for (i, &cu) in line.iter().enumerate() {
        state = match state {
            BetweenArgs => match cu {
                c if c == u16::from(b' ') => BetweenArgs,
                c if c == u16::from(b'"') => InArg(true),
                c if c == u16::from(b'\\') => Backslashes(1, false),
                c => {
                    push(&mut arg, c, false);
                    InArg(false)
                },
            },
            InArg(quoted) => match cu {
                c if c == u16::from(b'\\') => Backslashes(1, quoted),
                c if quoted && c == u16::from(b'"') => OnQuote,
                c if !quoted && c == u16::from(b'"') => InArg(true),
                c if !quoted && c == u16::from(b' ') => {
                    return (Some(arg), &line[i+1..]);
                },
                c => {
                    push(&mut arg, c, quoted);
                    InArg(quoted)
                },
            },
            OnQuote => match cu {
                c if c == u16::from(b'"') => {
                    // In quoted arg "" means literal quote and the end of the quoted string (but not arg)
                    push(&mut arg, u16::from(b'"'), true);
                    InArg(false)
                },
                c if c == u16::from(b' ') => {
                    return (Some(arg), &line[i+1..]);
                },
                c => {
                    push(&mut arg, c, false);
                    InArg(false)
                },
            },
            Backslashes(count, quoted) => match cu {
                c if c == u16::from(b'\\') => Backslashes(count + 1, quoted),
                c if c == u16::from(b'"') => {
                    // backslashes followed by a quotation mark are treated as pairs of protected backslashes
                    for _ in 0..count/2 {
                        push(&mut arg, u16::from(b'\\'), quoted);
                    }

                    if count & 1 != 0 {
                        // An odd number of backslashes is treated as followed by a protected quotation mark.
                        push(&mut arg, u16::from(b'"'), quoted);
                        InArg(quoted)
                    } else if quoted {
                        // An even number of backslashes is treated as followed by a word terminator.
                        return (Some(arg), &line[i+1..]);
                    } else {
                        InArg(quoted)
                    }
                },
                c => {
                    // A string of backslashes not followed by a quotation mark has no special meaning.
                    for _ in 0..count {
                        push(&mut arg, u16::from(b'\\'), quoted);
                    }
                    push(&mut arg, c, quoted);
                    InArg(quoted)
                },
            },
        }
    }
    let arg = match state {
        BetweenArgs => None,
        OnQuote | InArg(..) => Some(arg),
        Backslashes(count, quoted) => {
            // A string of backslashes not followed by a quotation mark has no special meaning.
            for _ in 0..count {
                push(&mut arg, u16::from(b'\\'), quoted);
            }

            Some(arg)
        },
    };
    (arg, &line[..0])
}
