#[cfg(feature = "locales")]
mod icu_impl {
    pub use icu::calendar::Date;
    pub use icu::datetime::DateTimeFormatter;
    pub use icu::datetime::fieldsets::builder::{DateFields, FieldSetBuilder, ZoneStyle};
    pub use icu::datetime::fieldsets::enums::CompositeFieldSet;
    pub use icu::datetime::options::{Length, SubsecondDigits, TimePrecision, YearStyle};
    pub use icu::locale::Locale;
    pub use icu::locale::extensions::unicode::{Value, key};
    pub use icu::time::zone::{IanaParser, UtcOffset};
    pub use icu::time::{Time, ZonedDateTime as IcuZonedDateTime};
}
#[cfg(feature = "locales")]
use icu_impl::*;
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
#[serde(untagged)]
#[cfg_attr(not(feature = "locales"), allow(dead_code))]
pub enum Locales {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum LocaleMatcher {
    Lookup,
    #[serde(rename = "best fit")]
    BestFit,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum FormatMatcher {
    Basic,
    #[serde(rename = "best fit")]
    BestFit,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum HourCycle {
    H11,
    H12,
    H23,
    H24,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DateTimeStyle {
    Full,
    Long,
    Medium,
    Short,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TextWidth {
    Long,
    Short,
    Narrow,
}

#[derive(Deserialize, Clone, Copy)]
pub enum Numeric2Digit {
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "2-digit")]
    TwoDigit,
}

#[derive(Deserialize, Clone, Copy)]
pub enum MonthFormat {
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "2-digit")]
    TwoDigit,
    #[serde(rename = "long")]
    Long,
    #[serde(rename = "short")]
    Short,
    #[serde(rename = "narrow")]
    Narrow,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TimeZoneNameStyle {
    Long,
    Short,
    ShortOffset,
    LongOffset,
    ShortGeneric,
    LongGeneric,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(feature = "locales"), allow(dead_code))]
pub struct ToLocaleStringOptions {
    #[allow(dead_code)]
    pub locale_matcher: Option<LocaleMatcher>,
    #[allow(dead_code)]
    pub format_matcher: Option<FormatMatcher>,
    pub time_zone: Option<String>,
    pub calendar: Option<String>,
    pub numbering_system: Option<String>,
    pub hour12: Option<bool>,
    pub hour_cycle: Option<HourCycle>,
    pub date_style: Option<DateTimeStyle>,
    pub time_style: Option<DateTimeStyle>,
    pub weekday: Option<TextWidth>,
    pub era: Option<TextWidth>,
    pub year: Option<Numeric2Digit>,
    pub month: Option<MonthFormat>,
    pub day: Option<Numeric2Digit>,
    pub day_period: Option<TextWidth>,
    pub hour: Option<Numeric2Digit>,
    pub minute: Option<Numeric2Digit>,
    pub second: Option<Numeric2Digit>,
    pub fractional_second_digits: Option<u8>,
    pub time_zone_name: Option<TimeZoneNameStyle>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DurationFormatStyle {
    Long,
    Short,
    Narrow,
    Digital,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DurationFieldStyle {
    Long,
    Short,
    Narrow,
    Numeric,
    #[serde(rename = "2-digit")]
    TwoDigit,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DurationFieldDisplay {
    Auto,
    Always,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(feature = "locales"), allow(dead_code))]
pub struct DurationFormatOptions {
    pub locale_matcher: Option<LocaleMatcher>,
    pub numbering_system: Option<String>,
    pub style: Option<DurationFormatStyle>,
    pub years: Option<DurationFieldStyle>,
    pub years_display: Option<DurationFieldDisplay>,
    pub months: Option<DurationFieldStyle>,
    pub months_display: Option<DurationFieldDisplay>,
    pub weeks: Option<DurationFieldStyle>,
    pub weeks_display: Option<DurationFieldDisplay>,
    pub days: Option<DurationFieldStyle>,
    pub days_display: Option<DurationFieldDisplay>,
    pub hours: Option<DurationFieldStyle>,
    pub hours_display: Option<DurationFieldDisplay>,
    pub minutes: Option<DurationFieldStyle>,
    pub minutes_display: Option<DurationFieldDisplay>,
    pub seconds: Option<DurationFieldStyle>,
    pub seconds_display: Option<DurationFieldDisplay>,
    pub milliseconds: Option<DurationFieldStyle>,
    pub milliseconds_display: Option<DurationFieldDisplay>,
    pub microseconds: Option<DurationFieldStyle>,
    pub microseconds_display: Option<DurationFieldDisplay>,
    pub nanoseconds: Option<DurationFieldStyle>,
    pub nanoseconds_display: Option<DurationFieldDisplay>,
    pub fractional_digits: Option<u8>,
}

#[cfg_attr(not(feature = "locales"), allow(dead_code))]
impl ToLocaleStringOptions {
    fn has_style(&self) -> bool {
        self.date_style.is_some() || self.time_style.is_some()
    }

    fn has_components(&self) -> bool {
        self.weekday.is_some()
            || self.era.is_some()
            || self.year.is_some()
            || self.month.is_some()
            || self.day.is_some()
            || self.day_period.is_some()
            || self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
            || self.time_zone_name.is_some()
    }

    fn validate(&self) -> Result<(), Error> {
        if let Some(d) = self.fractional_second_digits {
            if !(1..=3).contains(&d) {
                return Err("fractionalSecondDigits must be between 1 and 3".into());
            }
        }
        if self.has_style() && self.has_components() {
            return Err(
                "dateStyle and timeStyle cannot be used together with other date/time component options"
                    .into(),
            );
        }
        Ok(())
    }
}

#[cfg(feature = "locales")]
fn default_system_locale() -> Locale {
    sys_locale::get_locale()
        .and_then(|s| Locale::try_from_str(&s).ok())
        .unwrap_or_else(|| Locale::try_from_str("en").unwrap())
}

#[cfg(feature = "locales")]
fn resolve_locale(
    locales: Option<Locales>,
    options: &ToLocaleStringOptions,
) -> Result<Locale, Error> {
    let mut locale = match locales {
        Some(Locales::Single(tag)) => Locale::try_from_str(&tag)
            .map_err(|e| Error::Error(format!("Invalid locale tag '{}': {}", tag, e)))?,
        Some(Locales::Multiple(tags)) => {
            let mut chosen = None;
            for tag in &tags {
                let parsed = Locale::try_from_str(tag)
                    .map_err(|e| Error::Error(format!("Invalid locale tag '{}': {}", tag, e)))?;
                if chosen.is_none() {
                    chosen = Some(parsed);
                }
            }
            chosen.unwrap_or_else(default_system_locale)
        }
        None => default_system_locale(),
    };

    if let Some(cal) = &options.calendar {
        let val = Value::try_from_str(cal)
            .map_err(|e| Error::Error(format!("Invalid calendar '{}': {}", cal, e)))?;
        locale.extensions.unicode.keywords.set(key!("ca"), val);
    }

    if let Some(nu) = &options.numbering_system {
        let val = Value::try_from_str(nu)
            .map_err(|e| Error::Error(format!("Invalid numberingSystem '{}': {}", nu, e)))?;
        locale.extensions.unicode.keywords.set(key!("nu"), val);
    }

    if let Some(hc) = options.hour_cycle {
        let hc_str = match hc {
            HourCycle::H11 => "h11",
            HourCycle::H12 => "h12",
            HourCycle::H23 => "h23",
            HourCycle::H24 => "h24",
        };
        locale
            .extensions
            .unicode
            .keywords
            .set(key!("hc"), Value::try_from_str(hc_str).unwrap());
    } else if let Some(hour12) = options.hour12 {
        let hc_str = if hour12 { "h12" } else { "h23" };
        locale
            .extensions
            .unicode
            .keywords
            .set(key!("hc"), Value::try_from_str(hc_str).unwrap());
    }

    Ok(locale)
}

#[cfg(feature = "locales")]
fn build_field_set(options: &ToLocaleStringOptions) -> Result<CompositeFieldSet, Error> {
    let mut builder = FieldSetBuilder::default();

    if options.has_style() {
        if let Some(ds) = options.date_style {
            builder.date_fields = Some(match ds {
                DateTimeStyle::Full => DateFields::YMDE,
                DateTimeStyle::Long | DateTimeStyle::Medium | DateTimeStyle::Short => {
                    DateFields::YMD
                }
            });
            builder.length = Some(match ds {
                DateTimeStyle::Full | DateTimeStyle::Long => Length::Long,
                DateTimeStyle::Medium => Length::Medium,
                DateTimeStyle::Short => Length::Short,
            });
        }
        if let Some(ts) = options.time_style {
            builder.time_precision = Some(match ts {
                DateTimeStyle::Full | DateTimeStyle::Long | DateTimeStyle::Medium => {
                    TimePrecision::Second
                }
                DateTimeStyle::Short => TimePrecision::Minute,
            });
            if builder.length.is_none() {
                builder.length = Some(match ts {
                    DateTimeStyle::Full | DateTimeStyle::Long => Length::Long,
                    DateTimeStyle::Medium => Length::Medium,
                    DateTimeStyle::Short => Length::Short,
                });
            }
            match ts {
                DateTimeStyle::Full => builder.zone_style = Some(ZoneStyle::SpecificLong),
                DateTimeStyle::Long => builder.zone_style = Some(ZoneStyle::SpecificShort),
                DateTimeStyle::Medium | DateTimeStyle::Short => {}
            }
        }
        return builder
            .build_composite()
            .map_err(|e| Error::Error(format!("Invalid format options: {:?}", e)));
    }

    if !options.has_components() {
        // ECMA-402 default for Instant: numeric date + time
        builder.date_fields = Some(DateFields::YMD);
        builder.length = Some(Length::Short);
        builder.time_precision = Some(TimePrecision::Second);
        return builder
            .build_composite()
            .map_err(|e| Error::Error(format!("Invalid format options: {:?}", e)));
    }

    let has_y = options.year.is_some();
    let has_m = options.month.is_some();
    let has_d = options.day.is_some();
    let has_e = options.weekday.is_some();

    builder.date_fields = match (has_y, has_m, has_d, has_e) {
        (false, false, false, false) => None,
        (true, true, true, true) => Some(DateFields::YMDE),
        (true, true, true, false) => Some(DateFields::YMD),
        (false, true, true, true) => Some(DateFields::MDE),
        (false, true, true, false) => Some(DateFields::MD),
        (false, false, true, true) => Some(DateFields::DE),
        (false, false, true, false) => Some(DateFields::D),
        (false, false, false, true) => Some(DateFields::E),
        (true, true, false, false) => Some(DateFields::YM),
        (true, false, false, false) => Some(DateFields::Y),
        (false, true, false, false) => Some(DateFields::M),
        _ => Some(DateFields::YMD),
    };

    if has_y || options.era.is_some() {
        builder.year_style = Some(match (options.year, options.era) {
            (_, Some(_)) => YearStyle::WithEra,
            (Some(Numeric2Digit::TwoDigit), None) => YearStyle::Auto,
            (Some(Numeric2Digit::Numeric), None) | (None, None) => YearStyle::Full,
        });
    }

    if builder.date_fields.is_some() {
        builder.length = Some(
            if matches!(options.month, Some(MonthFormat::Long))
                || matches!(options.weekday, Some(TextWidth::Long))
            {
                Length::Long
            } else if matches!(options.month, Some(MonthFormat::Short))
                || matches!(options.weekday, Some(TextWidth::Short))
            {
                Length::Medium
            } else {
                Length::Short
            },
        );
    }

    let has_time = options.hour.is_some()
        || options.minute.is_some()
        || options.second.is_some()
        || options.fractional_second_digits.is_some()
        || options.day_period.is_some();

    if has_time {
        builder.time_precision = Some(if let Some(d) = options.fractional_second_digits {
            let subsec = match d {
                1 => SubsecondDigits::S1,
                2 => SubsecondDigits::S2,
                3 => SubsecondDigits::S3,
                _ => SubsecondDigits::S3,
            };
            TimePrecision::Subsecond(subsec)
        } else if options.second.is_some() {
            TimePrecision::Second
        } else if options.minute.is_some() {
            TimePrecision::Minute
        } else {
            TimePrecision::Hour
        });
    }

    builder.zone_style = options.time_zone_name.map(|z| match z {
        TimeZoneNameStyle::Long => ZoneStyle::SpecificLong,
        TimeZoneNameStyle::Short => ZoneStyle::SpecificShort,
        TimeZoneNameStyle::ShortOffset => ZoneStyle::LocalizedOffsetShort,
        TimeZoneNameStyle::LongOffset => ZoneStyle::LocalizedOffsetLong,
        TimeZoneNameStyle::ShortGeneric => ZoneStyle::GenericShort,
        TimeZoneNameStyle::LongGeneric => ZoneStyle::GenericLong,
    });

    builder
        .build_composite()
        .map_err(|e| Error::Error(format!("Invalid format options: {:?}", e)))
}

#[cfg(feature = "locales")]
pub fn format_instant(
    instant: &temporal_rs::Instant,
    locales: Option<Locales>,
    options: Option<ToLocaleStringOptions>,
) -> Result<String, Error> {
    let options = options.unwrap_or_default();
    options.validate()?;

    let locale = resolve_locale(locales, &options)?;
    let field_set = build_field_set(&options)?;

    let time_zone = match &options.time_zone {
        Some(tz) => temporal_rs::TimeZone::try_from_str(tz)?,
        None => temporal_rs::now::Now::new(temporal_rs::sys::LocalHostSystem).time_zone()?,
    };

    let zdt = instant.to_zoned_date_time_iso(time_zone)?;

    let date = Date::try_new_iso(zdt.year(), zdt.month(), zdt.day())
        .map_err(|e| Error::Error(format!("Failed to convert date: {}", e)))?;

    let subsec_ns = (zdt.millisecond() as u32) * 1_000_000
        + (zdt.microsecond() as u32) * 1_000
        + (zdt.nanosecond() as u32);
    let time = Time::try_new(zdt.hour(), zdt.minute(), zdt.second(), subsec_ns)
        .map_err(|e| Error::Error(format!("Failed to convert time: {}", e)))?;

    let tz_id = zdt.time_zone().identifier()?;
    let offset_secs = (zdt.offset_nanoseconds() / 1_000_000_000) as i32;
    let offset = UtcOffset::try_from_seconds(offset_secs)
        .map_err(|e| Error::Error(format!("Invalid UTC offset: {:?}", e)))?;

    let icu_tz = IanaParser::new().parse(&tz_id);
    let zone_info = icu_tz
        .with_offset(Some(offset))
        .at_date_time(icu::time::DateTime { date, time });

    let icu_zdt = IcuZonedDateTime {
        date,
        time,
        zone: zone_info,
    };

    let dtf = DateTimeFormatter::try_new(locale.into(), field_set)
        .map_err(|e| Error::Error(format!("Failed to create DateTimeFormatter: {}", e)))?;

    Ok(dtf.format(&icu_zdt).to_string())
}

#[cfg(feature = "locales")]
fn duration_display(
    value: DurationFieldDisplay,
) -> icu::experimental::duration::options::FieldDisplay {
    use icu::experimental::duration::options::FieldDisplay;
    match value {
        DurationFieldDisplay::Auto => FieldDisplay::Auto,
        DurationFieldDisplay::Always => FieldDisplay::Always,
    }
}

#[cfg(feature = "locales")]
fn calendar_duration_style(
    value: DurationFieldStyle,
) -> Result<icu::experimental::duration::options::YearStyle, Error> {
    use icu::experimental::duration::options::YearStyle;
    match value {
        DurationFieldStyle::Long => Ok(YearStyle::Long),
        DurationFieldStyle::Short => Ok(YearStyle::Short),
        DurationFieldStyle::Narrow => Ok(YearStyle::Narrow),
        _ => Err("numeric and 2-digit are not valid for years, months, weeks, or days".into()),
    }
}

#[cfg(feature = "locales")]
fn time_duration_style(
    value: DurationFieldStyle,
) -> icu::experimental::duration::options::HourStyle {
    use icu::experimental::duration::options::HourStyle;
    match value {
        DurationFieldStyle::Long => HourStyle::Long,
        DurationFieldStyle::Short => HourStyle::Short,
        DurationFieldStyle::Narrow => HourStyle::Narrow,
        DurationFieldStyle::Numeric => HourStyle::Numeric,
        DurationFieldStyle::TwoDigit => HourStyle::TwoDigit,
    }
}

#[cfg(feature = "locales")]
fn subsecond_duration_style(
    value: DurationFieldStyle,
) -> Result<icu::experimental::duration::options::MilliSecondStyle, Error> {
    use icu::experimental::duration::options::MilliSecondStyle;
    match value {
        DurationFieldStyle::Long => Ok(MilliSecondStyle::Long),
        DurationFieldStyle::Short => Ok(MilliSecondStyle::Short),
        DurationFieldStyle::Narrow => Ok(MilliSecondStyle::Narrow),
        DurationFieldStyle::Numeric => Ok(MilliSecondStyle::Numeric),
        DurationFieldStyle::TwoDigit => {
            Err("2-digit is not valid for milliseconds, microseconds, or nanoseconds".into())
        }
    }
}

#[cfg(feature = "locales")]
fn icu_duration(
    duration: &temporal_rs::Duration,
) -> Result<icu::experimental::duration::Duration, Error> {
    use icu::experimental::duration::{Duration, DurationSign};

    let magnitude = |value: i128| {
        u64::try_from(value.unsigned_abs())
            .map_err(|_| Error::Error("Duration field is out of range for ICU formatting".into()))
    };
    Ok(Duration {
        sign: if duration.sign() == temporal_rs::Sign::Negative {
            DurationSign::Negative
        } else {
            DurationSign::Positive
        },
        years: magnitude(i128::from(duration.years()))?,
        months: magnitude(i128::from(duration.months()))?,
        weeks: magnitude(i128::from(duration.weeks()))?,
        days: magnitude(i128::from(duration.days()))?,
        hours: magnitude(i128::from(duration.hours()))?,
        minutes: magnitude(i128::from(duration.minutes()))?,
        seconds: magnitude(i128::from(duration.seconds()))?,
        milliseconds: magnitude(i128::from(duration.milliseconds()))?,
        microseconds: magnitude(duration.microseconds())?,
        nanoseconds: magnitude(duration.nanoseconds())?,
    })
}

#[cfg(feature = "locales")]
pub fn format_duration(
    duration: &temporal_rs::Duration,
    locales: Option<Locales>,
    options: Option<DurationFormatOptions>,
) -> Result<String, Error> {
    use icu::experimental::duration::ValidatedDurationFormatterOptions;
    use icu::experimental::duration::options::{
        BaseStyle, DayStyle, DurationFormatterOptions, FractionalDigits, MonthStyle, WeekStyle,
    };

    let options = options.unwrap_or_default();
    let _locale_matcher = options.locale_matcher;
    if options.fractional_digits.is_some_and(|digits| digits > 9) {
        return Err("fractionalDigits must be between 0 and 9".into());
    }

    let locale_options = ToLocaleStringOptions {
        numbering_system: options.numbering_system.clone(),
        ..Default::default()
    };
    let locale = resolve_locale(locales, &locale_options)?;
    let mut formatter_options = DurationFormatterOptions::default();
    formatter_options.base = match options.style.unwrap_or(DurationFormatStyle::Short) {
        DurationFormatStyle::Long => BaseStyle::Long,
        DurationFormatStyle::Short => BaseStyle::Short,
        DurationFormatStyle::Narrow => BaseStyle::Narrow,
        DurationFormatStyle::Digital => BaseStyle::Digital,
    };
    formatter_options.fractional_digits = options
        .fractional_digits
        .map_or(FractionalDigits::ShowAll, FractionalDigits::Fixed);

    formatter_options.year = options.years.map(calendar_duration_style).transpose()?;
    formatter_options.year_visibility = options.years_display.map(duration_display);
    formatter_options.month = options
        .months
        .map(calendar_duration_style)
        .transpose()?
        .map(|style| match style {
            icu::experimental::duration::options::YearStyle::Long => MonthStyle::Long,
            icu::experimental::duration::options::YearStyle::Short => MonthStyle::Short,
            icu::experimental::duration::options::YearStyle::Narrow => MonthStyle::Narrow,
            _ => unreachable!(),
        });
    formatter_options.month_visibility = options.months_display.map(duration_display);
    formatter_options.week = options
        .weeks
        .map(calendar_duration_style)
        .transpose()?
        .map(|style| match style {
            icu::experimental::duration::options::YearStyle::Long => WeekStyle::Long,
            icu::experimental::duration::options::YearStyle::Short => WeekStyle::Short,
            icu::experimental::duration::options::YearStyle::Narrow => WeekStyle::Narrow,
            _ => unreachable!(),
        });
    formatter_options.week_visibility = options.weeks_display.map(duration_display);
    formatter_options.day = options
        .days
        .map(calendar_duration_style)
        .transpose()?
        .map(|style| match style {
            icu::experimental::duration::options::YearStyle::Long => DayStyle::Long,
            icu::experimental::duration::options::YearStyle::Short => DayStyle::Short,
            icu::experimental::duration::options::YearStyle::Narrow => DayStyle::Narrow,
            _ => unreachable!(),
        });
    formatter_options.day_visibility = options.days_display.map(duration_display);
    formatter_options.hour = options.hours.map(time_duration_style);
    formatter_options.hour_visibility = options.hours_display.map(duration_display);
    formatter_options.minute = options
        .minutes
        .map(|style| match time_duration_style(style) {
            icu::experimental::duration::options::HourStyle::Long => {
                icu::experimental::duration::options::MinuteStyle::Long
            }
            icu::experimental::duration::options::HourStyle::Short => {
                icu::experimental::duration::options::MinuteStyle::Short
            }
            icu::experimental::duration::options::HourStyle::Narrow => {
                icu::experimental::duration::options::MinuteStyle::Narrow
            }
            icu::experimental::duration::options::HourStyle::Numeric => {
                icu::experimental::duration::options::MinuteStyle::Numeric
            }
            icu::experimental::duration::options::HourStyle::TwoDigit => {
                icu::experimental::duration::options::MinuteStyle::TwoDigit
            }
            _ => unreachable!(),
        });
    formatter_options.minute_visibility = options.minutes_display.map(duration_display);
    formatter_options.second = options
        .seconds
        .map(|style| match time_duration_style(style) {
            icu::experimental::duration::options::HourStyle::Long => {
                icu::experimental::duration::options::SecondStyle::Long
            }
            icu::experimental::duration::options::HourStyle::Short => {
                icu::experimental::duration::options::SecondStyle::Short
            }
            icu::experimental::duration::options::HourStyle::Narrow => {
                icu::experimental::duration::options::SecondStyle::Narrow
            }
            icu::experimental::duration::options::HourStyle::Numeric => {
                icu::experimental::duration::options::SecondStyle::Numeric
            }
            icu::experimental::duration::options::HourStyle::TwoDigit => {
                icu::experimental::duration::options::SecondStyle::TwoDigit
            }
            _ => unreachable!(),
        });
    formatter_options.second_visibility = options.seconds_display.map(duration_display);
    formatter_options.millisecond = options
        .milliseconds
        .map(subsecond_duration_style)
        .transpose()?;
    formatter_options.millisecond_visibility = options.milliseconds_display.map(duration_display);
    formatter_options.microsecond = options
        .microseconds
        .map(subsecond_duration_style)
        .transpose()?
        .map(|style| match style {
            icu::experimental::duration::options::MilliSecondStyle::Long => {
                icu::experimental::duration::options::MicroSecondStyle::Long
            }
            icu::experimental::duration::options::MilliSecondStyle::Short => {
                icu::experimental::duration::options::MicroSecondStyle::Short
            }
            icu::experimental::duration::options::MilliSecondStyle::Narrow => {
                icu::experimental::duration::options::MicroSecondStyle::Narrow
            }
            icu::experimental::duration::options::MilliSecondStyle::Numeric => {
                icu::experimental::duration::options::MicroSecondStyle::Numeric
            }
            _ => unreachable!(),
        });
    formatter_options.microsecond_visibility = options.microseconds_display.map(duration_display);
    formatter_options.nanosecond = options
        .nanoseconds
        .map(subsecond_duration_style)
        .transpose()?
        .map(|style| match style {
            icu::experimental::duration::options::MilliSecondStyle::Long => {
                icu::experimental::duration::options::NanoSecondStyle::Long
            }
            icu::experimental::duration::options::MilliSecondStyle::Short => {
                icu::experimental::duration::options::NanoSecondStyle::Short
            }
            icu::experimental::duration::options::MilliSecondStyle::Narrow => {
                icu::experimental::duration::options::NanoSecondStyle::Narrow
            }
            icu::experimental::duration::options::MilliSecondStyle::Numeric => {
                icu::experimental::duration::options::NanoSecondStyle::Numeric
            }
            _ => unreachable!(),
        });
    formatter_options.nanosecond_visibility = options.nanoseconds_display.map(duration_display);

    let formatter_options = ValidatedDurationFormatterOptions::validate(formatter_options)
        .map_err(|e| Error::Error(format!("Invalid duration format options: {e}")))?;
    let formatter =
        icu::experimental::duration::DurationFormatter::try_new(locale.into(), formatter_options)
            .map_err(|e| Error::Error(format!("Failed to create DurationFormatter: {e}")))?;
    let duration = icu_duration(duration)?;
    Ok(writeable::Writeable::write_to_string(&formatter.format(&duration)).into_owned())
}

#[cfg(not(feature = "locales"))]
pub fn format_instant(
    _instant: &temporal_rs::Instant,
    _locales: Option<Locales>,
    _options: Option<ToLocaleStringOptions>,
) -> Result<String, Error> {
    Err("locales feature is not enabled".into())
}

#[cfg(not(feature = "locales"))]
pub fn format_duration(
    _duration: &temporal_rs::Duration,
    _locales: Option<Locales>,
    _options: Option<DurationFormatOptions>,
) -> Result<String, Error> {
    Err("locales feature is not enabled".into())
}
