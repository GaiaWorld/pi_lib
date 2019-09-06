use std::ffi::OsString;
use parser;
use std::fmt;

pub(crate) struct ArgOs {
    pub pattern: OsString,
    pub text: OsString,
    pub contains_glob: bool,
}

/// Iterator retuning glob-escaped arguments. Call `args()` to obtain it.
#[must_use]
pub(crate) struct GlobArgs<'a> {
    line: &'a [u16],
}

impl<'a> fmt::Debug for GlobArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        String::from_utf16_lossy(self.line).fmt(f)
    }
}

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

/// This is used only in tests on non-Windows
#[cfg(not(windows))]
trait LossyOsStringExt {
    fn from_wide(wide: &[u16]) -> OsString {
        OsString::from(String::from_utf16_lossy(wide))
    }
}

#[cfg(not(windows))]
impl LossyOsStringExt for OsString {}

impl<'a> Iterator for GlobArgs<'a> {
    type Item = ArgOs;
    fn next(&mut self) -> Option<Self::Item> {
        let state = (vec![], vec![], false);
        let (state, rest) = parser::next_arg(self.line, state, |&mut (ref mut arg, ref mut text, ref mut contains_glob), c, quoted| {
            text.push(c);
            match c as u8 {
                b'?' | b'*' | b'[' | b']' if c < 256 => {
                    if quoted {
                        arg.push(u16::from(b'['));
                        arg.push(c);
                        arg.push(u16::from(b']'));
                    } else {
                        arg.push(c);
                        *contains_glob = true;
                    }
                },
                _ => arg.push(c),
            };
        });
        self.line = rest;
        state.map(|(pattern, text, contains_glob)| ArgOs {
            pattern: OsString::from_wide(&pattern),
            text: OsString::from_wide(&text),
            contains_glob,
        })
    }
}

impl<'a> GlobArgs<'a> {
    /// UTF-16/UCS2 string from `GetCommandLineW`
    #[allow(dead_code)]
    pub(crate) fn new(line: &'a [u16]) -> Self {
        Self { line }
    }
}

