extern crate libc;

use std::fs;
use std::os::unix::prelude::*;

use self::libc::{timespec, time_t};

use FileTime;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod utimes;
        mod linux;
        pub use self::linux::*;
    } else if #[cfg(any(target_os = "android",
                        target_os = "solaris",
                        target_os = "emscripten",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd"))] {
        mod utimensat;
        pub use self::utimensat::*;
    } else {
        mod utimes;
        pub use self::utimes::*;
    }
}

#[allow(dead_code)]
fn to_timespec(ft: &Option<FileTime>) -> timespec {
    const UTIME_OMIT: i64 = 1073741822;

    if let &Some(ft) = ft {
        timespec {
            tv_sec: ft.seconds() as time_t,
            tv_nsec: ft.nanoseconds() as _,
        }
    } else {
        timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT as _,
        }
    }
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    FileTime {
        seconds: meta.mtime(),
        nanos: meta.mtime_nsec() as u32,
    }
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    FileTime {
        seconds: meta.atime(),
        nanos: meta.atime_nsec() as u32,
    }
}

pub fn from_creation_time(meta: &fs::Metadata) -> Option<FileTime> {
    macro_rules! birthtim {
        ($(($e:expr, $i:ident)),*) => {
            #[cfg(any($(target_os = $e),*))]
            fn imp(meta: &fs::Metadata) -> Option<FileTime> {
                $(
                    #[cfg(target_os = $e)]
                    use std::os::$i::fs::MetadataExt;
                )*
                Some(FileTime {
                    seconds: meta.st_birthtime(),
                    nanos: meta.st_birthtime_nsec() as u32,
                })
            }

            #[cfg(all($(not(target_os = $e)),*))]
            fn imp(_meta: &fs::Metadata) -> Option<FileTime> {
                None
            }
        }
    }

    birthtim! {
        ("bitrig", bitrig),
        ("freebsd", freebsd),
        ("ios", ios),
        ("macos", macos),
        ("openbsd", openbsd)
    }

    imp(meta)
}
