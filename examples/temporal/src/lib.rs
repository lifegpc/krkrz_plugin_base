use std::str::FromStr;

use krkrz_plugin_base::param::TypeError;
use krkrz_plugin_base::{tp_stub::*, *};
use serde::{Deserialize, Deserializer};
use temporal_rs::TemporalError;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(temporal);

mod locales;

struct Instant(temporal_rs::Instant);

enum Error {
    Temporal(TemporalError),
    Type(TypeError),
    Error(String),
    DerseError(de::DeserError),
}

impl From<TemporalError> for Error {
    fn from(value: TemporalError) -> Self {
        Self::Temporal(value)
    }
}

impl From<TypeError> for Error {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Error(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Error(value.to_owned())
    }
}

impl From<de::DeserError> for Error {
    fn from(value: de::DeserError) -> Self {
        Self::DerseError(value)
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> From<tp_stub::GetItemError<E>> for Error {
    fn from(value: tp_stub::GetItemError<E>) -> Self {
        Self::Error(format!("{}", value))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Temporal(t) => t.fmt(f),
            Self::Type(t) => t.fmt(f),
            Self::Error(t) => t.fmt(f),
            Self::DerseError(t) => t.fmt(f),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FractionalSecondDigits {
    #[serde(deserialize_with = "Self::deserialize_auto")]
    Auto,
    Digit(u8),
}

impl FractionalSecondDigits {
    fn to_precision(&self) -> Result<temporal_rs::parsers::Precision, Error> {
        match self {
            Self::Auto => Ok(temporal_rs::parsers::Precision::Auto),
            Self::Digit(d) => {
                if *d > 9 {
                    return Err("fractionalSecondDigits must be between 0 and 9".into());
                }
                Ok(temporal_rs::parsers::Precision::Digit(*d))
            }
        }
    }

    fn deserialize_auto<'de, D>(deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "auto" {
            Ok(())
        } else {
            Err(serde::de::Error::custom("expected \"auto\""))
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

impl RoundingMode {
    fn to_rounding_mode(&self) -> temporal_rs::options::RoundingMode {
        match self {
            Self::Ceil => temporal_rs::options::RoundingMode::Ceil,
            Self::Floor => temporal_rs::options::RoundingMode::Floor,
            Self::Expand => temporal_rs::options::RoundingMode::Expand,
            Self::Trunc => temporal_rs::options::RoundingMode::Trunc,
            Self::HalfCeil => temporal_rs::options::RoundingMode::HalfCeil,
            Self::HalfFloor => temporal_rs::options::RoundingMode::HalfFloor,
            Self::HalfExpand => temporal_rs::options::RoundingMode::HalfExpand,
            Self::HalfTrunc => temporal_rs::options::RoundingMode::HalfTrunc,
            Self::HalfEven => temporal_rs::options::RoundingMode::HalfEven,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Unit {
    Auto,
    #[serde(alias = "nanoseconds")]
    Nanosecond,
    #[serde(alias = "microseconds")]
    Microsecond,
    #[serde(alias = "milliseconds")]
    Millisecond,
    #[serde(alias = "seconds")]
    Second,
    #[serde(alias = "minutes")]
    Minute,
    #[serde(alias = "hours")]
    Hour,
    #[serde(alias = "days")]
    Day,
    #[serde(alias = "weeks")]
    Week,
    #[serde(alias = "months")]
    Month,
    #[serde(alias = "years")]
    Year,
}

impl Unit {
    fn to_unit(&self) -> temporal_rs::options::Unit {
        match self {
            Self::Auto => temporal_rs::options::Unit::Auto,
            Self::Nanosecond => temporal_rs::options::Unit::Nanosecond,
            Self::Microsecond => temporal_rs::options::Unit::Microsecond,
            Self::Millisecond => temporal_rs::options::Unit::Millisecond,
            Self::Second => temporal_rs::options::Unit::Second,
            Self::Minute => temporal_rs::options::Unit::Minute,
            Self::Hour => temporal_rs::options::Unit::Hour,
            Self::Day => temporal_rs::options::Unit::Day,
            Self::Week => temporal_rs::options::Unit::Week,
            Self::Month => temporal_rs::options::Unit::Month,
            Self::Year => temporal_rs::options::Unit::Year,
        }
    }
}

#[Tjs2Class]
#[tjs(new = "Temporal.Instant")]
impl Instant {
    fn new(nanoseconds: i128) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::try_new(nanoseconds)?))
    }

    #[tjs(static_method)]
    fn compare(instant1: &mut tTJSVariant, instant2: &mut tTJSVariant) -> Result<i32, Error> {
        let i1 = if Self::is_instance_of2(instant1) {
            Self::get_ref2(instant1).get_epoch_nanoseconds()
        } else {
            Self::from(instant1)?.get_epoch_nanoseconds()
        };
        let i2 = if Self::is_instance_of2(instant2) {
            Self::get_ref2(instant2).get_epoch_nanoseconds()
        } else {
            Self::from(instant2)?.get_epoch_nanoseconds()
        };
        Ok(std::cmp::Ord::cmp(&i1, &i2) as i8 as i32)
    }

    #[allow(non_upper_case_globals)]
    fn from(info: &mut tTJSVariant) -> Result<Self, Error> {
        match info.typ() {
            tTJSVariantType_tvtString => {
                let s = String::to_param(info)?;
                Ok(Self(temporal_rs::Instant::from_str(&s)?))
            }
            tTJSVariantType_tvtObject => {
                let obj = info.as_object_no_add_ref();
                if obj.is_null() {
                    throw_null_access();
                }
                if unsafe {
                    (*obj).is_instance_of(
                        0,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        tjs_w!("Date"),
                        obj,
                    )
                } == TJS_S_TRUE
                {
                    // is Date
                    let mut var = tTJSVariant::new();
                    let hr = unsafe {
                        (*obj).func_call(
                            0,
                            tjs_w!("getTime"),
                            std::ptr::null_mut(),
                            &mut var,
                            0,
                            std::ptr::null_mut(),
                            obj,
                        )
                    };
                    if TJS_FAILED(hr) {
                        return Err("failed to get time from Date.".into());
                    }
                    let time = i64::to_param(&mut var)?;
                    return Ok(Self(temporal_rs::Instant::from_epoch_milliseconds(time)?));
                }
                let hr = unsafe {
                    (*obj).is_instance_of(
                        0,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        tjs_w!("Instant"),
                        obj,
                    )
                };
                if hr != TJS_S_TRUE {
                    return Err(TypeError("Instant or Date").into());
                }
                let mut nano = tTJSVariant::new();
                let hr = unsafe {
                    (*obj).prop_get(
                        0,
                        tjs_w!("epochNanoseconds"),
                        std::ptr::null_mut(),
                        &mut nano,
                        obj,
                    )
                };
                if TJS_FAILED(hr) {
                    return Err("failed to get nanoseconds from Instant.".into());
                }
                let nanoseconds = i128::to_param(&mut nano)?;
                Ok(Self(temporal_rs::Instant::try_new(nanoseconds)?))
            }
            _ => Err(TypeError("string or Instant or Date").into()),
        }
    }

    #[tjs(case = camel)]
    fn from_epoch_milliseconds(epoch_milliseconds: i64) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::from_epoch_milliseconds(
            epoch_milliseconds,
        )?))
    }

    #[tjs(case = camel)]
    fn from_epoch_nanoseconds(epoch_nanoseconds: i128) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::try_new(epoch_nanoseconds)?))
    }

    #[tjs(case = camel)]
    fn get_epoch_milliseconds(&self) -> i64 {
        self.0.epoch_milliseconds()
    }

    #[tjs(case = camel)]
    fn get_epoch_nanoseconds(&self) -> i128 {
        self.0.epoch_nanoseconds().0
    }

    fn add(&self, duration: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let v = if Duration::is_instance_of2(duration) {
            let dur = Duration::get_ref2(duration);
            self.0.add(&dur.0)?
        } else {
            let dur = Duration::from(duration)?;
            self.0.add(&dur.0)?
        };
        let new_instant = Self(v);
        let n = Instant::create(new_instant);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    fn equals(&self, other: &mut tTJSVariant) -> Result<bool, Error> {
        let o = if Self::is_instance_of2(other) {
            Self::get_ref2(other).get_epoch_nanoseconds()
        } else {
            Self::from(other)?.get_epoch_nanoseconds()
        };
        Ok(self.get_epoch_nanoseconds() == o)
    }

    fn round(&self, options: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let mut smallest_unit = Unit::Auto;
        let mut inc = 1;
        let mut rounding_mode = RoundingMode::HalfExpand;
        if options.is_string() {
            smallest_unit = de::from(options)?;
        } else {
            let a = options.as_object_no_add_ref();
            if a.is_null() {
                return Err("options is not a object.".into());
            }
            let c = unsafe { &mut (*a) };
            c.with_get_serde("smallestUnit", |s| {
                smallest_unit = s;
            })?
            .ok_or_else(|| Error::from("smallestUnit must specified."))?;
            let mut t = c.get("roundingIncrement")?;
            if let Some(t) = t.as_mut() {
                inc = t.as_integer();
            }
            c.with_get_serde("roundingMode", |s| {
                rounding_mode = s;
            })?;
        }
        let mut opt = temporal_rs::options::RoundingOptions::default();
        opt.largest_unit = None;
        opt.smallest_unit = Some(smallest_unit.to_unit());
        opt.rounding_mode = Some(rounding_mode.to_rounding_mode());
        opt.increment = Some(temporal_rs::options::RoundingIncrement::try_new(
            inc as u32,
        )?);
        let re = Self(self.0.round(opt)?);
        let n = Instant::create(re);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    fn since(
        &self,
        other: &mut tTJSVariant,
        options: Option<&mut tTJSVariant>,
    ) -> Result<tTJSVariant, Error> {
        let o = if Self::is_instance_of2(other) {
            Self::get_ref2(other).0.clone()
        } else {
            Self::from(other)?.0
        };
        let opt = Self::_parse_diff_opt(options)?;
        let re = Duration(self.0.since(&o, opt)?);
        let n = Duration::create(re);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    fn subtract(&self, duration: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let v = if Duration::is_instance_of2(duration) {
            let dur = Duration::get_ref2(duration);
            self.0.subtract(&dur.0)?
        } else {
            let dur = Duration::from(duration)?;
            self.0.subtract(&dur.0)?
        };
        let new_instant = Self(v);
        let n = Instant::create(new_instant);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    #[tjs(rename = "toJSON")]
    #[inline(always)]
    fn to_json(&self, options: Option<&mut tTJSVariant>) -> Result<String, Error> {
        self.to_string(options)
    }

    #[tjs(case = camel, serde)]
    fn to_locale_string(
        &self,
        locales: Option<locales::Locales>,
        options: Option<locales::ToLocaleStringOptions>,
    ) -> Result<String, Error> {
        locales::format_instant(&self.0, locales, options)
    }

    #[tjs(case = camel)]
    fn to_string(&self, options: Option<&mut tTJSVariant>) -> Result<String, Error> {
        let mut fractional_second_digits = FractionalSecondDigits::Auto;
        let mut rounding_mode = RoundingMode::Trunc;
        let mut smallest_unit: Option<Unit> = None;
        let mut time_zone = None;
        let config_obj = match options.as_ref() {
            Some(c) => {
                let a = c.as_object_no_add_ref();
                if a.is_null() {
                    return Err("options is not a object.".into());
                }
                Some(unsafe { &mut (*a) })
            }
            None => None,
        };
        if let Some(c) = config_obj {
            c.with_get_serde("fractionalSecondDigits", |s| {
                fractional_second_digits = s;
            })?;
            c.with_get_serde("roundingMode", |s| {
                rounding_mode = s;
            })?;
            c.with_get_serde("smallestUnit", |s| {
                smallest_unit = Some(s);
            })?;
            let mut tz = c.get("timeZone")?;
            if let Some(tz) = tz.as_mut() {
                if tz.is_string() {
                    time_zone = Some(temporal_rs::TimeZone::try_from_str(&String::to_param(tz)?)?);
                } else if ZonedDateTime::is_instance_of2(tz) {
                    let zdt = ZonedDateTime::get_ref2(tz);
                    time_zone = Some(zdt.0.time_zone().clone());
                } else {
                    return Err(TypeError("string or ZonedDateTime").into());
                }
            }
        }
        Ok(self.0.to_ixdtf_string(
            time_zone,
            temporal_rs::options::ToStringRoundingOptions {
                precision: fractional_second_digits.to_precision()?,
                rounding_mode: Some(rounding_mode.to_rounding_mode()),
                smallest_unit: smallest_unit.map(|u| u.to_unit()),
            },
        )?)
    }

    #[tjs(rename = "toZonedDateTimeISO")]
    fn to_zoned_date_time_iso(&self, time_zone: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let tz;
        if time_zone.is_string() {
            tz = temporal_rs::TimeZone::try_from_str(&String::to_param(time_zone)?)?;
        } else if ZonedDateTime::is_instance_of2(time_zone) {
            let zdt = ZonedDateTime::get_ref2(time_zone);
            tz = zdt.0.time_zone().clone();
        } else {
            return Err(TypeError("string or ZonedDateTime").into());
        }
        let re = ZonedDateTime(self.0.to_zoned_date_time_iso(tz)?);
        let n = ZonedDateTime::create(re);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    fn until(
        &self,
        other: &mut tTJSVariant,
        options: Option<&mut tTJSVariant>,
    ) -> Result<tTJSVariant, Error> {
        let o = if Self::is_instance_of2(other) {
            Self::get_ref2(other).0.clone()
        } else {
            Self::from(other)?.0
        };
        let opt = Self::_parse_diff_opt(options)?;
        let re = Duration(self.0.until(&o, opt)?);
        let n = Duration::create(re);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    #[tjs(skip)]
    fn _parse_diff_opt(
        options: Option<&mut tTJSVariant>,
    ) -> Result<temporal_rs::options::DifferenceSettings, Error> {
        let mut smallest_unit = Unit::Nanosecond;
        let mut inc = 1;
        let mut rounding_mode = RoundingMode::HalfExpand;
        let mut largest_unit = Unit::Auto;
        if let Some(options) = options {
            let a = options.as_object_no_add_ref();
            if a.is_null() {
                return Err("options is not a object.".into());
            }
            let c = unsafe { &mut (*a) };
            c.with_get_serde("largestUnit", |s| {
                largest_unit = s;
            })?;
            let mut t = c.get("roundingIncrement")?;
            if let Some(t) = t.as_mut() {
                inc = t.as_integer();
            }
            c.with_get_serde("roundingMode", |s| {
                rounding_mode = s;
            })?;
            c.with_get_serde("smallestUnit", |s| {
                smallest_unit = s;
            })?;
        }
        let mut opt = temporal_rs::options::DifferenceSettings::default();
        opt.largest_unit = Some(largest_unit.to_unit());
        opt.smallest_unit = Some(smallest_unit.to_unit());
        opt.rounding_mode = Some(rounding_mode.to_rounding_mode());
        opt.increment = Some(temporal_rs::options::RoundingIncrement::try_new(
            inc as u32,
        )?);
        Ok(opt)
    }
}

impl Drop for Instant {
    fn drop(&mut self) {
        log!("Instant: dropped {:?}", self.0)
    }
}

struct Duration(temporal_rs::Duration);

#[derive(Default, Deserialize)]
struct DurationFromInfo {
    years: Option<i64>,
    months: Option<i64>,
    weeks: Option<i64>,
    days: Option<i64>,
    hours: Option<i64>,
    minutes: Option<i64>,
    seconds: Option<i64>,
    milliseconds: Option<i64>,
    microseconds: Option<i128>,
    nanoseconds: Option<i128>,
}

impl DurationFromInfo {
    fn is_empty(&self) -> bool {
        self.years.is_none()
            && self.months.is_none()
            && self.weeks.is_none()
            && self.days.is_none()
            && self.hours.is_none()
            && self.minutes.is_none()
            && self.seconds.is_none()
            && self.milliseconds.is_none()
            && self.microseconds.is_none()
            && self.nanoseconds.is_none()
    }

    fn into_duration(
        self,
        defaults: Option<&temporal_rs::Duration>,
    ) -> Result<temporal_rs::Duration, Error> {
        Ok(temporal_rs::Duration::new(
            self.years
                .unwrap_or_else(|| defaults.map_or(0, |d| d.years())),
            self.months
                .unwrap_or_else(|| defaults.map_or(0, |d| d.months())),
            self.weeks
                .unwrap_or_else(|| defaults.map_or(0, |d| d.weeks())),
            self.days
                .unwrap_or_else(|| defaults.map_or(0, |d| d.days())),
            self.hours
                .unwrap_or_else(|| defaults.map_or(0, |d| d.hours())),
            self.minutes
                .unwrap_or_else(|| defaults.map_or(0, |d| d.minutes())),
            self.seconds
                .unwrap_or_else(|| defaults.map_or(0, |d| d.seconds())),
            self.milliseconds
                .unwrap_or_else(|| defaults.map_or(0, |d| d.milliseconds())),
            self.microseconds
                .unwrap_or_else(|| defaults.map_or(0, |d| d.microseconds())),
            self.nanoseconds
                .unwrap_or_else(|| defaults.map_or(0, |d| d.nanoseconds())),
        )?)
    }
}

fn parse_duration_integer(value: &mut tTJSVariant) -> Result<i128, Error> {
    i128::to_param(value).map_err(Error::from)
}

fn parse_relative_to(value: &mut tTJSVariant) -> Result<temporal_rs::options::RelativeTo, Error> {
    use temporal_rs::options::RelativeTo;

    if PlainDate::is_instance_of2(value) {
        return Ok(RelativeTo::PlainDate(PlainDate::get_ref2(value).0.clone()));
    }
    if ZonedDateTime::is_instance_of2(value) {
        return Ok(RelativeTo::ZonedDateTime(
            ZonedDateTime::get_ref2(value).0.clone(),
        ));
    }
    if !value.is_string() {
        return Err(TypeError("string, PlainDate, or ZonedDateTime").into());
    }

    let text = String::to_param(value)?;
    let provider = &*temporal_rs::provider::COMPILED_TZ_PROVIDER;
    Ok(RelativeTo::try_from_str_with_provider(&text, provider)?)
}

fn parse_relative_to_option(
    object: &mut iTJSDispatch2,
) -> Result<Option<temporal_rs::options::RelativeTo>, Error> {
    let mut relative_to = object.get("relativeTo")?;
    match relative_to.as_mut() {
        Some(value) if !value.is_void() => Ok(Some(parse_relative_to(value)?)),
        _ => Ok(None),
    }
}

fn duration_variant(value: temporal_rs::Duration) -> tTJSVariant {
    let object = Duration::create(Duration(value));
    let result = tTJSVariant::from(object);
    unsafe { (*object).release() };
    result
}

#[Tjs2Class]
#[tjs(new = "Temporal.Duration")]
impl Duration {
    fn new(
        years: Option<i64>,
        months: Option<i64>,
        weeks: Option<i64>,
        days: Option<i64>,
        hours: Option<i64>,
        minutes: Option<i64>,
        seconds: Option<i64>,
        milliseconds: Option<i64>,
        microseconds: Option<i128>,
        nanoseconds: Option<i128>,
    ) -> Result<Self, Error> {
        Ok(Self(temporal_rs::Duration::new(
            years.unwrap_or(0),
            months.unwrap_or(0),
            weeks.unwrap_or(0),
            days.unwrap_or(0),
            hours.unwrap_or(0),
            minutes.unwrap_or(0),
            seconds.unwrap_or(0),
            milliseconds.unwrap_or(0),
            microseconds.unwrap_or(0),
            nanoseconds.unwrap_or(0),
        )?))
    }

    #[tjs(static_method)]
    fn compare(
        duration1: &mut tTJSVariant,
        duration2: &mut tTJSVariant,
        options: Option<&mut tTJSVariant>,
    ) -> Result<i32, Error> {
        let d1 = Self::from(duration1)?;
        let d2 = Self::from(duration2)?;
        let mut relative_to = None;
        if let Some(options) = options {
            let a = options.as_object_no_add_ref();
            if a.is_null() {
                return Err("options is not a object.".into());
            }
            let c = unsafe { &mut (*a) };
            relative_to = parse_relative_to_option(c)?;
        }
        Ok(d1.0.compare(&d2.0, relative_to)? as i8 as i32)
    }

    #[allow(non_upper_case_globals)]
    fn from(info: &mut tTJSVariant) -> Result<Self, Error> {
        match info.typ() {
            tTJSVariantType_tvtObject => {
                if Duration::is_instance_of2(info) {
                    let inf = Duration::get_ref2(info);
                    return Ok(Self(inf.0.clone()));
                }
                let fields = de::from::<DurationFromInfo>(info)?;
                if fields.is_empty() {
                    return Err(TypeError("Duration-like object with at least one field").into());
                }
                Ok(Self(fields.into_duration(None)?))
            }
            tTJSVariantType_tvtString => {
                let s = String::to_param(info)?;
                Ok(Self(temporal_rs::Duration::from_str(&s)?))
            }
            _ => Err(TypeError("string or Duration or Dict").into()),
        }
    }

    fn abs(&self) -> Result<tTJSVariant, Error> {
        Ok(duration_variant(self.0.abs()))
    }

    fn add(&self, other: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let re = if Self::is_instance_of2(other) {
            let dur = Self::get_ref2(other);
            self.0.add(&dur.0)?
        } else {
            let dur = Self::from(other)?;
            self.0.add(&dur.0)?
        };
        Ok(duration_variant(re))
    }

    fn negated(&self) -> Result<tTJSVariant, Error> {
        Ok(duration_variant(self.0.negated()))
    }

    fn get_years(&self) -> i64 {
        self.0.years()
    }

    fn get_months(&self) -> i64 {
        self.0.months()
    }

    fn get_weeks(&self) -> i64 {
        self.0.weeks()
    }

    fn get_days(&self) -> i64 {
        self.0.days()
    }

    fn get_hours(&self) -> i64 {
        self.0.hours()
    }

    fn get_minutes(&self) -> i64 {
        self.0.minutes()
    }

    fn get_seconds(&self) -> i64 {
        self.0.seconds()
    }

    fn get_milliseconds(&self) -> i64 {
        self.0.milliseconds()
    }

    fn get_microseconds(&self) -> i128 {
        self.0.microseconds()
    }

    fn get_nanoseconds(&self) -> i128 {
        self.0.nanoseconds()
    }

    fn get_sign(&self) -> i32 {
        self.0.sign() as i8 as i32
    }

    fn get_blank(&self) -> bool {
        self.0.is_zero()
    }

    fn with(&self, info: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let fields = de::from::<DurationFromInfo>(info)?;
        if fields.is_empty() {
            return Err(TypeError("Duration-like object with at least one field").into());
        }
        Ok(duration_variant(fields.into_duration(Some(&self.0))?))
    }

    fn subtract(&self, other: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let re = if Self::is_instance_of2(other) {
            self.0.subtract(&Self::get_ref2(other).0)?
        } else {
            self.0.subtract(&Self::from(other)?.0)?
        };
        Ok(duration_variant(re))
    }

    fn round(&self, round_to: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let mut largest_unit = None;
        let mut smallest_unit = None;
        let mut rounding_mode = None;
        let mut increment = None;
        let relative_to;

        if round_to.is_string() {
            smallest_unit = Some(de::from::<Unit>(round_to)?.to_unit());
            relative_to = None;
        } else {
            let object = round_to.as_object_no_add_ref();
            if object.is_null() {
                return Err(TypeError("string or object").into());
            }
            let object = unsafe { &mut (*object) };
            object.with_get_serde("largestUnit", |unit: Unit| {
                largest_unit = Some(unit.to_unit());
            })?;
            relative_to = parse_relative_to_option(object)?;
            let mut value = object.get("roundingIncrement")?;
            if let Some(value) = value.as_mut().filter(|value| !value.is_void()) {
                let number = parse_duration_integer(value)?;
                let number =
                    u32::try_from(number).map_err(|_| Error::from(TemporalError::range()))?;
                increment = Some(temporal_rs::options::RoundingIncrement::try_new(number)?);
            }
            object.with_get_serde("roundingMode", |mode: RoundingMode| {
                rounding_mode = Some(mode.to_rounding_mode());
            })?;
            object.with_get_serde("smallestUnit", |unit: Unit| {
                smallest_unit = Some(unit.to_unit());
            })?;
        }

        let mut round_options = temporal_rs::options::RoundingOptions::default();
        round_options.largest_unit = largest_unit;
        round_options.smallest_unit = smallest_unit;
        round_options.rounding_mode = rounding_mode;
        round_options.increment = increment;
        let result = self.0.round(round_options, relative_to)?;
        Ok(duration_variant(result))
    }

    fn total(&self, total_of: &mut tTJSVariant) -> Result<f64, Error> {
        let (unit, relative_to) = if total_of.is_string() {
            (de::from::<Unit>(total_of)?.to_unit(), None)
        } else {
            let object = total_of.as_object_no_add_ref();
            if object.is_null() {
                return Err(TypeError("string or object").into());
            }
            let object = unsafe { &mut (*object) };
            let mut unit = None;
            object
                .with_get_serde("unit", |value: Unit| {
                    unit = Some(value.to_unit());
                })?
                .ok_or_else(|| Error::from("unit must be specified."))?;
            (unit.unwrap(), parse_relative_to_option(object)?)
        };

        Ok(self.0.total(unit, relative_to)?.as_inner())
    }

    #[tjs(case = camel, serde)]
    fn to_locale_string(
        &self,
        locales: Option<locales::Locales>,
        options: Option<locales::DurationFormatOptions>,
    ) -> Result<String, Error> {
        locales::format_duration(&self.0, locales, options)
    }

    #[tjs(case = camel)]
    fn to_string(&self, options: Option<&mut tTJSVariant>) -> Result<String, Error> {
        let mut precision = FractionalSecondDigits::Auto;
        let mut rounding_mode = RoundingMode::Trunc;
        let mut smallest_unit = None;
        if let Some(options) = options {
            let object = options.as_object_no_add_ref();
            if object.is_null() {
                return Err(TypeError("object").into());
            }
            let object = unsafe { &mut (*object) };
            object.with_get_serde("fractionalSecondDigits", |value| precision = value)?;
            object.with_get_serde("roundingMode", |value| rounding_mode = value)?;
            object.with_get_serde("smallestUnit", |value: Unit| {
                smallest_unit = Some(value.to_unit());
            })?;
        }
        Ok(self
            .0
            .as_temporal_string(temporal_rs::options::ToStringRoundingOptions {
                precision: precision.to_precision()?,
                smallest_unit,
                rounding_mode: Some(rounding_mode.to_rounding_mode()),
            })?)
    }

    #[tjs(rename = "toJSON")]
    fn to_json(&self) -> Result<String, Error> {
        self.to_string(None)
    }
}

impl Drop for Duration {
    fn drop(&mut self) {
        log!("Duration: dropped {:?}", self.0)
    }
}

struct Now {}

#[Tjs2Class]
impl Now {
    #[tjs(static_method)]
    fn instant() -> Result<tTJSVariant, Error> {
        let now = temporal_rs::now::Now::new(temporal_rs::sys::LocalHostSystem).instant()?;
        let n = Instant::create(Instant(now));
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    #[tjs(static_method, case = camel)]
    fn time_zone_id() -> Result<String, Error> {
        let now = temporal_rs::now::Now::new(temporal_rs::sys::LocalHostSystem);
        let tz = now.time_zone()?;
        Ok(tz.identifier()?)
    }
}

struct PlainDate(temporal_rs::PlainDate);

#[Tjs2Class]
#[tjs(new = "Temporal.PlainDate")]
impl PlainDate {
    fn new(year: i64, month: i64, day: i64, calendar: Option<String>) -> Result<Self, Error> {
        let year = year as i32;
        let month = month as u8;
        let day = day as u8;
        if let Some(c) = calendar {
            let c = temporal_rs::Calendar::try_from_utf8(c.as_bytes())?;
            Ok(Self(temporal_rs::PlainDate::new(year, month, day, c)?))
        } else {
            Ok(Self(temporal_rs::PlainDate::new_iso(year, month, day)?))
        }
    }
}

struct ZonedDateTime(temporal_rs::ZonedDateTime);

#[Tjs2Class]
#[tjs(new = "Temporal.ZonedDateTime")]
impl ZonedDateTime {
    fn new(
        epoch_nanoseconds: i128,
        time_zone: String,
        calendar: Option<String>,
    ) -> Result<Self, Error> {
        let calendar = match calendar {
            Some(s) => temporal_rs::Calendar::try_from_utf8(s.as_bytes())?,
            None => temporal_rs::Calendar::ISO,
        };
        let time_zone = temporal_rs::TimeZone::try_from_str(&time_zone)?;
        Ok(Self(temporal_rs::ZonedDateTime::try_new(
            epoch_nanoseconds,
            time_zone,
            calendar,
        )?))
    }
}

struct Temporal {}

#[Tjs2Class]
impl Temporal {
    #[tjs(static_member, case = pascal)]
    fn instant() -> tTJSVariant {
        let class = Instant::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn duration() -> tTJSVariant {
        let class = Duration::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn now() -> tTJSVariant {
        let class = Now::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn plain_date() -> tTJSVariant {
        let class = PlainDate::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn zoned_date_time() -> tTJSVariant {
        let class = ZonedDateTime::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system-unwind" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let temporal = Temporal::create_native_class().1;
    register_var!(temporal);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system-unwind" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[temporal]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(temporal);
    log!("[temporal]unlinked plugin");
    unsafe { TVPUninitImportStub() };
    0
}
