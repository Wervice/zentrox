/// Parsing and creating cronjobs whilst trying to comply with most cron as well as anacron variants and keeping the
/// file and a reasonable complexity.
use std::fmt::Display;
use std::fs;
use std::process::{Command, Stdio};
use std::usize;

use regex::Regex;
use serde::Serialize;
use whoami::username;

/// The enumeration Interval is used to denote an Interval present in (ana)cron.
/// The enum variants are pretty self-explanatory.
#[allow(unused)]
#[derive(Debug, Copy, Clone, Serialize)]
pub enum Interval {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Annually,
    Yearly,
    Reboot,
}

/// The enumeration Digit is used represent the annotation of cron times in memory.
/// Cron uses values like "1-10" ,"2,4,3", "10/3" to represent intervals or future repetitive events.
/// Any: Any value is accepted (e.g.: * * * * * “At every minute.”)
/// Range(usize, usize): All values between point a and b (e.g. * 1-4 * * * “At every minute past every hour from 1 through 4.”)
/// List(Vec<usize>): A list of values (* 1,4 * * * "At every minute past hour 1 and 4.”)
/// Value(usize): A specific single value (* 4 * * * “At every minute past hour 4.”)
/// Repeating(String, usize):
///     Repeat the value a through b times until the limit (e.g. 59 for minutes starting at 0) is reached.
///     This is hard to calculate as a is a string. This is done to account for compositions like
///     */2 without turning Digit into a recursive enumeration.
/// Composed(String):
///     Composed accounts for all Digit representations that could not be matched to any other
///     Digit variant otherwise. As Digit representations can be recursive/complex, this is
///     required to keep this code simple.
#[allow(unused)]
#[derive(Debug, Serialize)]
pub enum Digit {
    Any,
    Range(usize, usize),
    List(Vec<usize>),
    Value(usize),
    Repeating(String, usize),
    Composed(String),
}

/// All months supported by cron. Digit can be used as months can also be expressed using numbers
/// (1-12) as well as names (jan-dec)
#[allow(unused)]
#[derive(Debug, Serialize)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
    Digit(Digit),
}

/// All days of the week. Digit can be used as months can also be expressed using numbers (0-6) as
/// well as names (sun-sat).
#[allow(unused)]
#[derive(Debug, Serialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    Digit(Digit),
}

/// The user to list/create cron jobs for. Specific selects a specific user while Current
/// automatically gets converted to the current active user when transformed using the Display
/// trait.
#[derive(PartialEq, Eq, Clone)]
pub enum User {
    Specific(String),
    Current,
}

impl Display for Interval {
    // Translates the enum variants of Interval into the representations for anacron.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Hourly => f.write_str("@hourly"),
            Self::Daily => f.write_str("@daily"),
            Self::Weekly => f.write_str("@weekly"),
            Self::Monthly => f.write_str("@monthly"),
            Self::Yearly => f.write_str("@yearly"),
            Self::Annually => f.write_str("@annually"),
            Self::Reboot => f.write_str("@reboot"),
        }
    }
}

impl Display for Digit {
    // Displays a Digit as a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => f.write_str("*"),
            Self::List(l) => {
                for ele in l {
                    let _ = f.write_str(ele.to_string().as_str());
                    let _ = f.write_str(",");
                }
                Ok(())
            }
            Self::Range(a, b) => f.write_str(format!("{a}-{b}").as_str()),
            Self::Value(v) => f.write_str(v.to_string().as_str()),
            Self::Repeating(s, r) => f.write_str(format!("{s}/{r}").as_str()),
            Self::Composed(s) => f.write_str(s.as_str()),
        }
    }
}

impl Display for Month {
    // Assigns every month a number. Numbers are used instead of names as it appears that month
    // names are not always supported by cron.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::January => f.write_str("1"),
            Self::February => f.write_str("2"),
            Self::March => f.write_str("3"),
            Self::April => f.write_str("4"),
            Self::May => f.write_str("5"),
            Self::June => f.write_str("6"),
            Self::July => f.write_str("7"),
            Self::August => f.write_str("8"),
            Self::September => f.write_str("9"),
            Self::October => f.write_str("10"),
            Self::November => f.write_str("11"),
            Self::December => f.write_str("12"),
            Self::Digit(d) => f.write_str(d.to_string().as_str()),
        }
    }
}

impl Display for DayOfWeek {
    // Assigns every month a number. Numbers are used instead of names as it appears that day
    // names are not always supported by cron.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DayOfWeek::Sunday => f.write_str("0"),
            DayOfWeek::Monday => f.write_str("1"),
            DayOfWeek::Tuesday => f.write_str("2"),
            DayOfWeek::Wednesday => f.write_str("3"),
            DayOfWeek::Thursday => f.write_str("4"),
            DayOfWeek::Friday => f.write_str("5"),
            DayOfWeek::Saturday => f.write_str("6"),
            Self::Digit(d) => f.write_str(d.to_string().as_str()),
        }
    }
}

impl Display for User {
    // Turns User into a string. User::Current is *not* predictable. It automatically adapts to the
    // current user when executed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            User::Specific(s) => f.write_str(s.as_str()),
            User::Current => f.write_str(username().as_str()),
        }
    }
}

#[derive(Debug)]
pub enum CronCreationError {
    CronReadingError,
    CronWritingError,
    CronFileCommandError,
    BadCrontabStatus,
    CronDenied,
}

pub fn write_cronfile(content: String, user: User) -> Option<()> {
    let random_uuid = uuid::Uuid::new_v4().to_string();
    let mut tmp_p = std::env::temp_dir();
    tmp_p.push(random_uuid);
    let pad = if !content.ends_with('\n') { "\n" } else { "" };
    let _ = fs::write(&tmp_p, format!("{}{}", content, pad));
    let mut c = Command::new("crontab");
    if user != User::Current {
        c.arg("-u");
        c.arg(user.to_string());
    }
    c.stdin(Stdio::null());
    c.stdout(Stdio::null());
    c.arg(tmp_p.to_str().unwrap());
    let x = c.output();
    let _ = fs::remove_file(tmp_p);

    if !x.unwrap().status.success() {
        return None;
    }

    return Some(());
}

fn get_cron_contents(user: User) -> Option<String> {
    let mut c = Command::new("crontab");
    c.stdin(Stdio::null());
    if user != User::Current {
        c.arg("-u");
        c.arg(user.to_string());
    }
    c.arg("-l");
    let x = c.output();

    match x {
        Ok(v) => {
            let err = v.stderr;
            let out = v.stdout;
            let status = v.status;
            if !status.success() || !err.is_empty() {
                return None;
            }
            let out_decoded = String::from_utf8(out);
            Some(out_decoded.unwrap().to_string())
        }
        Err(_) => return None,
    }
}

fn get_cron_lines(user: User) -> Option<Vec<String>> {
    match get_cron_contents(user) {
        Some(v) => Some(v.lines().map(String::from).collect::<Vec<String>>()),
        None => None,
    }
}

fn crontab_exists(user: User) -> bool {
    let mut c = Command::new("crontab");
    if user != User::Current {
        c.arg("-u");
        c.arg(user.to_string());
    }
    c.arg("-l");
    let x = c.output();

    match x {
        Ok(v) => {
            let err = v.stderr;
            let status = v.status;
            return status.success() && err.is_empty();
        }
        Err(_) => return false,
    }
}

/// Creates a completly new cron job for a given user.
/// This function does not verify input values.
pub fn create_new_specific_cronjob(
    job: SpecificCronJob,
    user: User,
) -> Result<String, CronCreationError> {
    let prompt = format!(
        "{} {} {} {} {} {}",
        job.minute.to_string(),
        job.hour.to_string(),
        job.day_of_month.to_string(),
        job.month.to_string(),
        job.day_of_week.to_string(),
        job.command.to_string(),
    );

    if crontab_exists(user.clone()) {
        match get_cron_contents(user.clone()) {
            Some(cont) => {
                let pad = if cont.ends_with("\n") {
                    ""
                } else if !cont.is_empty() {
                    "\n"
                } else {
                    ""
                };
                let new_cont = format!("{cont}{pad}{prompt}\n");
                match write_cronfile(new_cont.clone(), user) {
                    Some(_) => Ok(prompt),
                    None => Err(CronCreationError::CronWritingError),
                }
            }
            None => Err(CronCreationError::CronReadingError),
        }
    } else {
        match write_cronfile(prompt.clone(), user) {
            Some(_) => Ok(prompt),
            None => Err(CronCreationError::CronWritingError),
        }
    }
}

/// Creates a completly new cron job for a given user.
/// This function does not verify input values.
pub fn create_new_interval_cronjob(
    job: IntervalCronJob,
    user: User,
) -> Result<String, CronCreationError> {
    let prompt = format!("{} {}", job.interval.to_string(), job.command);

    if crontab_exists(user.clone()) {
        match get_cron_contents(user.clone()) {
            Some(cont) => {
                let pad = if cont.ends_with("\n") {
                    ""
                } else if !cont.is_empty() {
                    "\n"
                } else {
                    ""
                };
                let new_cont = format!("{cont}{pad}{prompt}\n");

                match write_cronfile(new_cont.clone(), user) {
                    Some(_) => Ok(prompt),
                    None => Err(CronCreationError::CronWritingError),
                }
            }
            None => Err(CronCreationError::CronReadingError),
        }
    } else {
        match write_cronfile(prompt.clone(), user) {
            Some(_) => Ok(prompt),
            None => Err(CronCreationError::CronWritingError),
        }
    }
}

impl Interval {
    fn is<T: Display>(v: T) -> bool {
        let intervals = [
            "@hourly",
            "@daily",
            "@weekly",
            "@monthly",
            "@yearly",
            "@annually",
            "@reboot",
        ];
        intervals.contains(&v.to_string().as_str())
    }
}

impl From<&str> for Interval {
    fn from(value: &str) -> Self {
        match value {
            "@hourly" => Self::Hourly,
            "@daily" => Self::Daily,
            "@weekly" => Self::Weekly,
            "@monthly" => Self::Monthly,
            "@yearly" => Self::Yearly,
            "@annually" => Self::Annually,
            "@reboot" => Self::Reboot,
            _ => panic!("Unknown cron job interval {}", value),
        }
    }
}

impl From<&str> for Digit {
    fn from(value: &str) -> Self {
        let mut modifiers = vec!['*', ',', '-', '/'];
        let mut matches = 0_i32;
        for c in value.chars() {
            if modifiers.contains(&c) {
                let index = modifiers.iter().position(|x| *x == c).unwrap();
                modifiers.remove(index);
                matches += 1;
            }
        }
        if matches >= 2 {
            return Self::Composed(value.to_string());
        }

        if value == "*" {
            Digit::Any
        } else if value.contains(",") {
            let v = value
                .split(",")
                .filter(|v| *v != "")
                .map(|v| v.parse::<usize>().unwrap())
                .collect();
            Digit::List(v)
        } else if value.contains("-") {
            let s = value
                .split("-")
                .map(|x| x.parse::<usize>().unwrap())
                .collect::<Vec<usize>>();
            Digit::Range(s[0], s[1])
        } else if value.contains("/") {
            let s = value.split("/").collect::<Vec<&str>>();
            Digit::Repeating(s[0].to_string(), s[1].parse::<usize>().unwrap())
        } else {
            let p = value.parse::<usize>();
            match p {
                Ok(pv) => return Self::Value(pv),
                Err(_) => return Self::Composed(value.to_string()),
            }
        }
    }
}

impl From<&str> for Month {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "jan" | "january" | "1" => Self::January,
            "feb" | "february" | "2" => Self::February,
            "mar" | "march" | "3" => Self::March,
            "apr" | "april" | "4" => Self::April,
            "may" | "5" => Self::May,
            "jun" | "june" | "6" => Self::June,
            "jul" | "july" | "7" => Self::July,
            "aug" | "august" | "8" => Self::August,
            "sep" | "september" | "9" => Self::September,
            "oct" | "october" | "10" => Self::October,
            "nov" | "november" | "11" => Self::November,
            "dec" | "december" | "12" => Self::December,
            v => Self::Digit(Digit::from(v)),
        }
    }
}

impl From<&str> for DayOfWeek {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "sun" | "sunday" | "0" => Self::Sunday,
            "mon" | "monday" | "1" => Self::Monday,
            "tue" | "tuesday" | "2" => Self::Tuesday,
            "wed" | "wednesday" | "3" => Self::Wednesday,
            "thu" | "thursday" | "4" => Self::Thursday,
            "fri" | "friday" | "5" => Self::Friday,
            "sat" | "saturday" | "6" => Self::Saturday,
            v => Self::Digit(Digit::from(v)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IntervalCronJob {
    pub interval: Interval,
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct SpecificCronJob {
    pub minute: Digit,
    pub hour: Digit,
    pub day_of_month: Digit,
    pub month: Month,
    pub day_of_week: DayOfWeek,
    pub command: String,
}

#[derive(Debug, Serialize)]
pub enum CronJob {
    Interval(IntervalCronJob),
    Specific(SpecificCronJob),
}

#[derive(Debug, Serialize)]
pub enum CronListingError {
    CronReadingError,
    NoCronFile,
}

fn cronjob_line_from_to(l: String, n: usize) -> String {
    let re = Regex::new(r"\s+");
    l.trim()[re.unwrap().find_iter(l.trim()).nth(n).unwrap().start()..]
        .trim_start()
        .to_string()
}

pub fn list_cronjobs(user: User) -> Result<Vec<CronJob>, CronListingError> {
    if !crontab_exists(user.clone()) {
        return Err(CronListingError::NoCronFile);
    }

    match get_cron_lines(user) {
        Some(lines) => {
            let mut jobs = Vec::new();
            for l in lines {
                let re = Regex::new(r"\s+").unwrap();
                let segments = re.split(l.trim()).collect::<Vec<&str>>();
                if Interval::is(segments[0]) {
                    if segments.len() < 2 {
                        continue;
                    }
                    let j = IntervalCronJob {
                        interval: Interval::from(segments[0]),
                        command: cronjob_line_from_to(l, 0),
                    };
                    jobs.push(CronJob::Interval(j))
                } else {
                    if segments.len() < 6 {
                        continue;
                    }
                    let minute = Digit::from(segments[0]);
                    let hour = Digit::from(segments[1]);
                    let day_of_month = Digit::from(segments[2]);
                    let month = Month::from(segments[3]);
                    let day_of_week = DayOfWeek::from(segments[4]);
                    let j = SpecificCronJob {
                        minute,
                        hour,
                        day_of_month,
                        day_of_week,
                        month,
                        command: cronjob_line_from_to(l, 4),
                    };
                    jobs.push(CronJob::Specific(j));
                }
            }
            return Ok(jobs);
        }
        None => return Err(CronListingError::CronReadingError),
    }
}

pub enum CronDeletionError {
    NoCronFile,
    IndexOutOfRange,
}

pub fn delete_specific_cronjob(target_index: u32, user: User) -> Result<(), CronDeletionError> {
    let mut lines;
    match get_cron_lines(user.clone()) {
        Some(v) => lines = v,
        None => return Err(CronDeletionError::NoCronFile),
    };

    let mut current_index = 0;
    let mut file_line_index = 0;

    for l in lines.clone() {
        let re = Regex::new(r"\s+").unwrap();
        let segments = re.split(l.trim()).collect::<Vec<&str>>();

        // Skip processing the line but aknowledge its existance if the line is a comment, interval
        // or invalid length.
        if segments.len() < 6 || l.starts_with("@") || l.starts_with("#") {
            file_line_index += 1;
            continue;
        }

        if current_index == target_index {
            lines.remove(file_line_index as usize);
        }
        current_index += 1;
        file_line_index += 1;
    }

    let mut s = String::new();
    for l in lines {
        s = format!("{s}{l}\n");
    }

    let _ = write_cronfile(s, user);

    return Ok(());
}

pub fn delete_interval_cronjob(target_index: u32, user: User) -> Result<(), CronDeletionError> {
    let mut lines;
    match get_cron_lines(user.clone()) {
        Some(v) => lines = v,
        None => return Err(CronDeletionError::NoCronFile),
    };

    let mut current_index = 0;
    let mut file_line_index = 0;

    for l in lines.clone() {
        let re = Regex::new(r"\s+").unwrap();
        let segments = re.split(l.trim()).collect::<Vec<&str>>();

        // Skip processing the line but aknowledge its existance if the line is a comment, interval
        // or invalid length.
        if segments.len() < 2
            || !l.starts_with("@")
            || !Interval::is(segments[0].trim())
            || l.starts_with("#")
        {
            file_line_index += 1;
            continue;
        }

        if current_index == target_index {
            lines.remove(file_line_index as usize);
        }
        current_index += 1;
        file_line_index += 1;
    }

    let mut s = String::new();
    for l in lines {
        s = format!("{s}{l}\n");
    }

    let _ = write_cronfile(s, user);

    return Ok(());
}
