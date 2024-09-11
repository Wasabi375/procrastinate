#[macro_export]
macro_rules! check_key_arg_doc {
    () => {
        "Only check the procrastination with this key.
Check for all if this is left empty."
    };
}

#[macro_export]
macro_rules! local_arg_doc {
    () => {
        "If this is set, this will check for procrastinations
in the current working directory (at \"procrastinate.ron\")
and not the user data directory.

This take precedence over `file`. Therefor if this is set the `file`
argument is ignored."
    };
}

#[macro_export]
macro_rules! file_arg_doc {
    () => {
        "Check for procrastinations in the given file.

This is ignored if `local` is set."
    };
}

const DELAY_TIMING_ARG_DOC: &str = "DELAY: a combination of the following intervals.
    Any interval must be in the format \"<n><tag>\" without a space between
    the number and tag. There must a a space between intervals.
    
    The tags are (year, y), (months, M), (weeks, w), (days, d), (hours, h),
    (min, m), (sec, s).

    e.g: 5m 3s
         1M 2d 7m";

pub const ONCE_TIMING_ARG_DOC: &str = constcat::concat!(
    "Can be either an Instant or a Delay.

INSTANT: can be one of the following
    today
    tomorrow
    Day of Month: \"dom 12\" => 12th day in the current or next month
        - can be followd by a time [h:m[:s]], e.g \"dom 15 7:42\"
    Day of Week: monday, tuesday, etc
        - can be followd by a time [h:m[:s]], e.g \"monday 13:12:11\"
    Any Date: \"y-M-d[ h:m[:s]]\"
              \"d-M[ h:m[:s]]\"
    Any Month: january, february, etc

",
    DELAY_TIMING_ARG_DOC
);

pub const REPEAT_TIMING_ARG_DOC: &str = constcat::concat!(
    "Can be either an Instant or a Delay.

INSTANT: Can be one of the following
    daily 
        - can be optionally be followed by a time [h:m[:s]], e.g \"daily 10:11\"
    day of week: monday, tuesday, etc
        - can be optionally be followed by a time [h:m[:s]], e.g \"friday 16:20\"
    monthly <day>
        - can be optionally be followed by a time [h:m[:s]], e.g \"monthly 5 10:11\"

",
    DELAY_TIMING_ARG_DOC
);
