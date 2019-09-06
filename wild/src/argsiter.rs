use globiter::*;
use std::ffi::OsString;
use glob;
use std::fmt;

/// Windows replacement for `std::env::ArgsOs`
#[cfg_attr(test, allow(dead_code))]
pub struct ArgsOs {
    pub(crate) args: Option<GlobArgs<'static>>,
    pub(crate) current_arg_globs: Option<glob::Paths>,
}

/// Windows replacement for `std::env::Args`
pub struct Args {
    pub(crate) iter: ArgsOs,
}

fn first_non_error<T,E,I>(iter: &mut I) -> Option<T> where I: Iterator<Item=Result<T,E>> {
    loop {
        match iter.next() {
            Some(Ok(item)) => return Some(item),
            None => return None,
            Some(Err(_)) => {},
        }
    }
}

impl Iterator for Args {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|s| s.to_string_lossy().to_string())
    }
}

impl Iterator for ArgsOs {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        let glob_options = glob::MatchOptions { case_sensitive: false, ..Default::default() };
        match self.current_arg_globs.as_mut().and_then(first_non_error) {
            Some(path) => Some(path.into_os_string()),
            None => match self.args {
                Some(ref mut args) => match args.next() {
                    // lossy: https://github.com/rust-lang-nursery/glob/issues/23
                    Some(arg) => if arg.contains_glob {
                        match glob::glob_with(&arg.pattern.to_string_lossy(), glob_options) {
                            Ok(mut glob_iter) => {
                                let first_glob = first_non_error(&mut glob_iter);
                                self.current_arg_globs = Some(glob_iter);
                                match first_glob {
                                    Some(path) => Some(path.into_os_string()),
                                    None => {
                                        // non-matching patterns are passed as regular strings
                                        self.current_arg_globs = None;
                                        Some(arg.text)
                                    },
                                }
                            }
                            Err(_) => {
                                // Invalid patterns are passed as regular strings
                                Some(arg.text)
                            },
                        }
                    } else {
                        // valid, but non-wildcard args passed as is, in order to avoid normalizing slashes
                        Some(arg.text)
                    },
                    None => None, // end of args
                },
                None => None, // error: no args available at all
            },
        }
    }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.iter.fmt(f)
    }
}

impl fmt::Debug for ArgsOs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.args.as_ref().map(|args| args.fmt(f))
            .unwrap_or_else(|| "".fmt(f))
    }
}


#[test]
fn finds_cargo_toml() {
    let cmd = "foo.exe _not_?a?_[f]ilename_ \"_not_?a?_[p]attern_\" Cargo.tom?".chars().map(|c| c as u16).collect::<Vec<_>>();
    let args = GlobArgs::new(Box::leak(cmd.into_boxed_slice()));
    let iter = Args {
        iter: ArgsOs {
            args: Some(args),
            current_arg_globs: None,
        },
    };
    assert_eq!("\"foo.exe _not_?a?_[f]ilename_ \\\"_not_?a?_[p]attern_\\\" Cargo.tom?\"", format!("{:?}", iter));
    let args: Vec<_> = iter.collect();
    assert_eq!(4, args.len());
    assert_eq!("foo.exe", &args[0]);
    assert_eq!("_not_?a?_[f]ilename_", &args[1]);
    assert_eq!("_not_?a?_[p]attern_", &args[2]);
    assert_eq!("Cargo.toml", &args[3]);
}

#[test]
fn unquoted_slashes_unchanged() {
    let cmd = r#"foo.exe //// .. ./ \\\\"#.chars().map(|c| c as u16).collect::<Vec<_>>();
    let args = GlobArgs::new(Box::leak(cmd.into_boxed_slice()));
    let iter = Args {
        iter: ArgsOs {
            args: Some(args),
            current_arg_globs: None,
        },
    };
    let args: Vec<_> = iter.collect();
    assert_eq!(5, args.len());
    assert_eq!("foo.exe", &args[0]);
    assert_eq!("////", &args[1]);
    assert_eq!("..", &args[2]);
    assert_eq!("./", &args[3]);
    assert_eq!(r#"\\\\"#, &args[4]);
}

#[test]
fn finds_readme_case_insensitive() {
    let cmd = "foo.exe _not_?a?_[f]ilename_ \"_not_?a?_[p]attern_\" read*.MD".chars().map(|c| c as u16).collect::<Vec<_>>();
    let args = GlobArgs::new(Box::leak(cmd.into_boxed_slice()));
    let iter = ArgsOs {
        args: Some(args),
        current_arg_globs: None,
    };
    let args: Vec<_> = iter.map(|c| c.to_string_lossy().to_string()).collect();
    assert_eq!(4, args.len());
    assert_eq!("foo.exe", &args[0]);
    assert_eq!("_not_?a?_[f]ilename_", &args[1]);
    assert_eq!("_not_?a?_[p]attern_", &args[2]);
    assert_eq!("README.md", &args[3]);
}
