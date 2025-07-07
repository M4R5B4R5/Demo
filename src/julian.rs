use rust_decimal::Decimal;
use rust_decimal::dec;
use rust_decimal::prelude::*;
use std::convert::TryFrom;
use std::error::Error;
use std::ops::Sub;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WeekDay {
    Sunday = 0,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Clone, Copy)]
pub enum WeekDayError {
    InvalidDayNumber,
    NonIntegerDecimal,
}

impl TryFrom<Decimal> for WeekDay {
    type Error = WeekDayError;

    fn try_from(d: Decimal) -> Result<Self, Self::Error> {
        let day = d.to_u8().ok_or(WeekDayError::NonIntegerDecimal)?;

        match day {
            1 => Ok(Self::Monday),
            2 => Ok(Self::Tuesday),
            3 => Ok(Self::Wednesday),
            4 => Ok(Self::Thursday),
            5 => Ok(Self::Friday),
            6 => Ok(Self::Saturday),
            0 => Ok(Self::Sunday),
            _ => Err(WeekDayError::InvalidDayNumber),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calendar {
    Gregorian,
    Julian,
}

#[derive(Debug)]
pub enum CalendarDateError {
    InvalidJulianDay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JulianDay {
    pub day: Decimal
}

impl From<CalendarDate> for JulianDay {
    /// Converts a CalendarDate into a JulianDay.
    /// 
    /// All valid CalendarDate objects can be converted into their corresponding JulianDay\
    /// **NOTE:** The reverse is NOT always true. 
    fn from(cd: CalendarDate) -> Self {
        JulianDay::from(&cd)
    }
}

impl From<&CalendarDate> for JulianDay {
    /// Converts a &CalendarDate into a JulianDay.
    fn from(cd: &CalendarDate) -> Self {
        let mut y = cd.y;
        let mut m = cd.m;
        let d = cd.d;

        if cd.m == 1 || cd.m == 2 {
            y = y - 1;
            m = m + 12;
        }

        let b = match cd.get_calendar() {
            Calendar::Gregorian => {
                let a = (Decimal::from(y) / dec!(100.0)).floor();
                dec!(2.0) - a + (a / dec!(4.0)).floor()
            },
            Calendar::Julian => dec!(0.0)
        };

        let j = (dec!(365.25) * (Decimal::from(y) + dec!(4716.0))).floor() + (dec!(30.6001) * (Decimal::from(m) + dec!(1.0))).floor() + d + b - dec!(1524.5);
        JulianDay::new(j)
    }
}

impl JulianDay {
    pub fn new(day: Decimal) -> Self {
        Self { day }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CalendarDate {
    y: i32,
    m: u8,
    d: Decimal,
}

impl CalendarDate {
    /// Does not validate input. The year, month and day MUST be a valid date.
    /// Refer to your local calendar if uncertain
    pub fn new(y: i32, m: u8, d: Decimal) -> Self {
        Self { y, m, d }
    }

    /// Determines if this calendar date falls on a leap year.
    /// 
    /// **NOTE**: The way a leap year is calculated depends on the calendar in use at the time.
    /// - If ``CalendarDate`` occurs strictly before 1582 October 15th, leap years will be calculated according to the Julian Calendar
    /// - If ``CalendarDate`` occurs during or after 1582 October 15th, leap years will be calculated according to the Gregorian Calendar
    pub fn leap_year(&self) -> bool {
        return match self.get_calendar() {
            Calendar::Gregorian => {
                self.y % 4 == 0
            },
            Calendar::Julian => {
                (self.y % 4 == 0) && ((self.y % 100 != 0) || (self.y % 400 == 0))
            }
        }
    }

    /// Returns the days between two CalendarDate objects.\
    /// Defined as: ``|lhs - rhs|``.
    pub fn days_between(lhs: &CalendarDate, rhs: &CalendarDate) -> Decimal {
        return Self::difference(lhs, rhs).abs();
    }

    /// Returns the day of the week corresponding to this CalendarDate.
    /// 
    /// **NOTE:** The week was not modified in any way by the Gregorian reform of the Julian calendar.\
    /// Thus, in 1582, ``Thursday October 4`` was followed by ``Friday October 15``.
    pub fn day_of_the_week(&self) -> WeekDay {
        let date_0hr = CalendarDate {y: self.y, m: self.m, d: self.d.round()};
        let jd = JulianDay::from(date_0hr);
        println!("{}", jd.day);

        let day = (jd.day + dec!(1.5)) % dec!(7);

        WeekDay::try_from(day).unwrap()
    }

    /// Returns an integer between and including 1 and 365 (or 366 if date corresponds to a leap year).\
    /// This represents the current day as an offset of the current year.
    pub fn day_of_the_year(&self) -> i32 {
        let k = Decimal::from(match self.leap_year() {
            true => 1,
            false => 2,
        });

        let m_d = Decimal::from(self.m);
        let d_d = Decimal::from(self.d);

        let n = ((dec!(275) * m_d) / dec!(9)).floor() - k * ((m_d + dec!(9)) / dec!(12)).floor() + d_d - dec!(30);
        return n.to_i32().unwrap()
    }

    /// Returns the difference between two CalendarDate objects.\
    /// Defined as: ``lhs - rhs``
    pub fn difference(lhs: &CalendarDate, rhs: &CalendarDate) -> Decimal {
        let lhs_jd = JulianDay::from(lhs);
        let rhs_jd = JulianDay::from(rhs);
        return lhs_jd.day - rhs_jd.day;
    }

    /// Determines what calendar system the current CalendarDate falls under
    pub fn get_calendar(&self) -> Calendar {
        let year_is_julian = self.y < 1582;
        let month_is_julian = self.y == 1582 && self.m < 10;
        let day_is_julian = self.y == 1582 && self.m == 10 && self.d < dec!(15.0);

        return if year_is_julian || month_is_julian || day_is_julian {
            Calendar::Julian
        } else {
            Calendar::Gregorian
        }
    }
}

impl TryFrom<JulianDay> for CalendarDate {
    type Error = CalendarDateError;

    /// Taken from "Calculation of the Calendar Date from the JD"
    /// 
    /// **NOTE:** A valid julian day does not neccessarily correspond to a valid calendar date
    /// We require jd >= 0 for the conversion to be successful
    fn try_from(j: JulianDay) -> Result<Self, Self::Error> {
        if j.day < Decimal::ZERO {
            return Err(CalendarDateError::InvalidJulianDay)
        }

        let jd = j.day + dec!(0.5);
        
        let z = jd.floor();
        let f = jd - z;
    
        let a = if z < dec!(2299161.0) {
            z
        } else {
            let alpha = ((z - dec!(1867216.25)) / dec!(36524.25)).floor();
            z + dec!(1.0) + alpha - (alpha / dec!(4.0)).floor()
        };
    
        let b = a + dec!(1524.0);
        let c = ((b - dec!(122.1)) / dec!(365.25)).floor();
        let d = (dec!(365.25) * c).floor();
        let e = ((b - d) / dec!(30.6001)).floor();
    
        let day = b - d - (dec!(30.6001) * e).floor() + f;
    
        let month = if e < dec!(14.0) {
            e - dec!(1.0)
        } else {
            e - dec!(13.0)
        };
    
        let year = if month > dec!(2.0) {
            c - dec!(4716.0)
        } else {
            c - dec!(4715.0)
        };
    
        Ok(CalendarDate::new(year.trunc().to_i32().unwrap(), month.trunc().to_u8().unwrap(), day))
    }
}

#[cfg(test)]
mod tests {
    use crate::julian::*;

    #[test]
    fn get_calendar_test() {
        let date = CalendarDate::new(1957, 10, dec!(4.81));
        assert_eq!(date.get_calendar(), Calendar::Gregorian);
        
        let date = CalendarDate::new(333, 1, dec!(27.5));
        assert_eq!(date.get_calendar(), Calendar::Julian);

        let date = CalendarDate::new(1582, 10, dec!(14.9));
        assert_eq!(date.get_calendar(), Calendar::Julian);

        let date = CalendarDate::new(1582, 10, dec!(15.0));
        assert_eq!(date.get_calendar(), Calendar::Gregorian);
    }

    #[test]
    fn julian_day_test() {
        // Example 7.a
        let date = CalendarDate::new(1957, 10, dec!(4.81));
        assert_eq!(JulianDay::from(date).day, dec!(2436116.31));

        // Example 7.b
        let date = CalendarDate::new(333, 1, dec!(27.5));
        assert_eq!(JulianDay::from(date).day, dec!(1842713.0));

        // Gregorian dates
        let g1 = CalendarDate::new(2000, 1, dec!(1.5));
        let g2 = CalendarDate::new(1999, 1, dec!(1.0));
        let g3 = CalendarDate::new(1987, 1, dec!(27.0));
        let g4 = CalendarDate::new(1600, 12, dec!(31.0));

        assert_eq!(JulianDay::from(g1).day, dec!(2451545.0));
        assert_eq!(JulianDay::from(g2).day, dec!(2451179.5));
        assert_eq!(JulianDay::from(g3).day, dec!(2446822.5));
        assert_eq!(JulianDay::from(g4).day, dec!(2305812.5));

        // Julian dates
        let j1 = CalendarDate::new(837, 4, dec!(10.3));
        let j2 = CalendarDate::new(-123, 12, dec!(31.0));
        let j3 = CalendarDate::new(-1000, 7, dec!(12.5));
        let j4 = CalendarDate::new(-4712, 1, dec!(1.5));

        assert_eq!(JulianDay::from(j1).day, dec!(2026871.8));
        assert_eq!(JulianDay::from(j2).day, dec!(1676496.5));
        assert_eq!(JulianDay::from(j3).day, dec!(1356001.0));
        assert_eq!(JulianDay::from(j4).day, dec!(0.0));
    }

    #[test]
    fn test_get_calendar_date() {
        let jd = JulianDay::new(dec!(2436116.31));
        let cd = CalendarDate::try_from(jd).unwrap();

        assert_eq!(cd, CalendarDate::new(1957, 10, dec!(4.81)));
    }

    #[test]
    fn test_interval_between() {
        let first = CalendarDate::new(1910, 4, dec!(20));
        let second = CalendarDate::new(1986, 2, dec!(9));
        
        assert_eq!(CalendarDate::days_between(&first, &second), dec!(27689));
    }

    #[test]
    fn test_day_of_the_week() {
        let date = CalendarDate::new(1954, 6, dec!(30));
        assert_eq!(date.day_of_the_week(), WeekDay::Wednesday);
    }

    #[test]
    fn test_day_of_the_year() {
        let date1 = CalendarDate::new(1978, 11, dec!(14));
        let date2 = CalendarDate::new(1988, 4, dec!(22));

        assert_eq!(date1.day_of_the_year(), 318);
        assert_eq!(date2.day_of_the_year(), 113);
    }
}