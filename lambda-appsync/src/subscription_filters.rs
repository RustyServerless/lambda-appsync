//! GraphQL subscription filter implementation for AWS AppSync
//!
//! This module provides types and abstractions for building type-safe GraphQL
//! subscription filters according to the AWS AppSync specification. The filters
//! are used to control which events are delivered to subscribed clients based
//! on the event payloads.
//!
//! The module enforces AWS AppSync's filter constraints at compile time, including:
//! - Maximum depth of 5 levels for nested field paths
//! - Maximum 256 character length for field paths
//! - `in` and `notIn` operators accept up to 5 values in an array
//! - `containsAny` operator accepts up to 20 values in an array
//!
//! # Examples
//!
//! Simple field equality filter:
//! ```
//! # use lambda_appsync::{subscription_filters::FieldPath, AppsyncError};
//! # fn example() -> Result<(), AppsyncError> {
//! let filter = FieldPath::new("user.name")?.eq("example");
//! # Ok(())
//! # }
//! ```
//!
//! Complex filter group with AND/OR logic:
//! ```
//! # use lambda_appsync::{subscription_filters::{FieldPath, Filter, FilterGroup}, AppsyncError};
//! # fn example() -> Result<FilterGroup, AppsyncError> {
//! // The FilterGroup combines Filter elements with OR logic
//! // This means the filter will match if ANY of the Filter conditions are true
//! let group = FilterGroup::from([
//!     // First filter - combines conditions with AND logic:
//!     // - user.role must equal "admin" AND
//!     // - user.age must be greater than 21
//!     Filter::from([
//!         FieldPath::new("user.role")?.eq("admin"),
//!         FieldPath::new("user.age")?.gt(21)
//!     ]),
//!     // Second filter - also uses AND logic between its conditions:
//!     // - user.role must equal "moderator" AND
//!     // - user.permissions must contain either "moderate" or "review"
//!     Filter::from([
//!         FieldPath::new("user.role")?.eq("moderator"),
//!         FieldPath::new("user.permissions")?.contains_any(["moderate", "review"])
//!     ])
//!     // Final logic:
//!     // (role="admin" AND age>21) OR (role="moderator" AND permissions∩["moderate","review"]≠∅)
//! ]);
//! # Ok(group)
//! # }
//! ```
//!
//! Array operators with size limits:
//! ```
//! # use lambda_appsync::{subscription_filters::FieldPath, AppsyncError};
//! # fn example() -> Result<(), AppsyncError> {
//! // IN operator (max 5 values)
//! let roles = FieldPath::new("user.role")?.in_values(["admin", "mod", "user"]);
//!
//! // ContainsAny operator (max 20 values)
//! let perms = FieldPath::new("user.permissions")?
//!     .contains_any(["read", "write", "delete", "admin"]);
//! # Ok(())
//! # }
//! ```

use serde::Serialize;

use crate::{
    AWSDate, AWSDateTime, AWSEmail, AWSPhone, AWSTime, AWSTimestamp, AWSUrl, AppsyncError, ID,
};

/// Marker trait for types that can be used as values in subscription filter operators.
///
/// This trait is implemented for numeric types ([`u8`], [`i8`], [`u16`], [`i16`], [`u32`],
/// [`i32`], [`u64`], [`i64`], [`u128`], [`i128`], [`f32`], [`f64`]), [`String`], [`str`],
/// and all AppSync scalar types ([`ID`], [`AWSEmail`], [`AWSUrl`], [`AWSDate`], [`AWSTime`],
/// [`AWSPhone`], [`AWSDateTime`], [`AWSTimestamp`]).
///
/// It is used as a bound on the comparison operators of [`FieldPath`] that accept numeric
/// or string values (e.g., [`FieldPath::le`], [`FieldPath::gt`], [`FieldPath::contains`]).
/// Boolean values are intentionally excluded from this trait — use [`IFSBValueMarker`] for
/// operators that also accept booleans (e.g., [`FieldPath::eq`], [`FieldPath::ne`]).
///
/// This trait is sealed: it cannot be implemented outside of this crate.
pub trait IFSValueMarker: private::Sealed + Serialize {
    /// Convert the value to a serde_json::Value
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("cannot fail for IFSValueMarker types")
    }
}

/// Marker trait for types that can be used as values in equality and inequality filter operators.
///
/// This trait extends the set of valid filter value types beyond [`IFSValueMarker`] by also
/// including [`bool`]. It is implemented for all numeric types, [`String`], [`str`], [`bool`],
/// and all AppSync scalar types ([`ID`], [`AWSEmail`], [`AWSUrl`], [`AWSDate`], [`AWSTime`],
/// [`AWSPhone`], [`AWSDateTime`], [`AWSTimestamp`]).
///
/// It is used as a bound on [`FieldPath::eq`] and [`FieldPath::ne`], which are the only
/// operators that accept boolean values in addition to numbers and strings.
///
/// This trait is sealed: it cannot be implemented outside of this crate.
pub trait IFSBValueMarker: private::Sealed + Serialize {
    /// Convert the value to a serde_json::Value
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("cannot fail for IFSBValueMarker types")
    }
}

/// Sealed trait module to prevent external implementations of marker traits.
mod private {
    pub trait Sealed {}
}

/// Implements one or more marker traits for a list of types in a single invocation.
macro_rules! impl_markers {
    (nested $tr:ty, ($($t:ty),+)) => {
        $(impl $tr for $t {})+
    };
    ($($tr:ty),+| $t:tt) => {
        $(impl_markers!(nested $tr, $t);)+
    }
}
impl_markers!(
    IFSBValueMarker,
    private::Sealed
        | (
            u8,
            i8,
            u16,
            i16,
            u32,
            i32,
            u64,
            i64,
            u128,
            i128,
            f32,
            f64,
            bool,
            String,
            &str,
            ID,
            AWSEmail,
            AWSUrl,
            AWSDate,
            AWSTime,
            AWSPhone,
            AWSDateTime,
            AWSTimestamp
        )
);
impl_markers!(
    IFSValueMarker
        | (
            u8,
            i8,
            u16,
            i16,
            u32,
            i32,
            u64,
            i64,
            u128,
            i128,
            f32,
            f64,
            String,
            &str,
            ID,
            AWSEmail,
            AWSUrl,
            AWSDate,
            AWSTime,
            AWSPhone,
            AWSDateTime,
            AWSTimestamp
        )
);

/// Fixed-size vector for operators with size limits
#[derive(Debug, Clone, PartialEq)]
pub struct FixedVec<T, const N: usize>([Option<T>; N]);

impl<T: Serialize, const N: usize> Serialize for FixedVec<T, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.0.iter().flatten())
    }
}
impl<T: IFSValueMarker, const N: usize> FixedVec<T, N> {
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("cannot fail for IFSValueMarker types")
    }
}

/// A vector limited to 5 elements for In/NotIn operators
type InVec<T> = FixedVec<T, 5>;

/// A vector limited to 20 elements for ContainsAny operator
type ContainsAnyVec<T> = FixedVec<T, 20>;

/// Implements `From<[T; N]>` for [`FixedVec<T, M>`] for each supported array length N ≤ M.
macro_rules! impl_from_array {
    (none 5) => {
        [None, None, None, None, None]
    };
    (none 10) => {
        [None, None, None, None, None, None, None, None, None, None]
    };
    (none 20) => {
        [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None,
        ]
    };
    ($m:tt; $n:literal; $(($idx:tt, $v:ident)),*) => {
        impl<T> From<[T; $n]> for FixedVec<T, $m> {
            fn from([$($v),*]: [T; $n]) -> Self {
                let mut slice = impl_from_array!(none $m);
                $((slice[$idx]).replace($v);)*
                Self(slice)
            }
        }
    };
}
impl_from_array!(5; 1; (0, v1));
impl_from_array!(5; 2; (0, v1), (1, v2));
impl_from_array!(5; 3; (0, v1), (1, v2), (2, v3));
impl_from_array!(5; 4; (0, v1), (1, v2), (2, v3), (3, v4));
impl_from_array!(5; 5; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5));
impl_from_array!(10; 1; (0, v1));
impl_from_array!(10; 2; (0, v1), (1, v2));
impl_from_array!(10; 3; (0, v1), (1, v2), (2, v3));
impl_from_array!(10; 4; (0, v1), (1, v2), (2, v3), (3, v4));
impl_from_array!(10; 5; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5));
impl_from_array!(10; 6; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6));
impl_from_array!(10; 7; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7));
impl_from_array!(10; 8; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8));
impl_from_array!(10; 9; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9));
impl_from_array!(10; 10; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10));
impl_from_array!(20; 1; (0, v1));
impl_from_array!(20; 2; (0, v1), (1, v2));
impl_from_array!(20; 3; (0, v1), (1, v2), (2, v3));
impl_from_array!(20; 4; (0, v1), (1, v2), (2, v3), (3, v4));
impl_from_array!(20; 5; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5));
impl_from_array!(20; 6; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6));
impl_from_array!(20; 7; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7));
impl_from_array!(20; 8; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8));
impl_from_array!(20; 9; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9));
impl_from_array!(20; 10; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10));
impl_from_array!(20; 11; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11));
impl_from_array!(20; 12; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12));
impl_from_array!(20; 13; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13));
impl_from_array!(20; 14; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14));
impl_from_array!(20; 15; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14), (14, v15));
impl_from_array!(20; 16; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14), (14, v15), (15, v16));
impl_from_array!(20; 17; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14), (14, v15), (15, v16), (16, v17));
impl_from_array!(20; 18; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14), (14, v15), (15, v16), (16, v17), (17, v18));
impl_from_array!(20; 19; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14), (14, v15), (15, v16), (16, v17), (17, v18), (18, v19));
impl_from_array!(20; 20; (0, v1), (1, v2), (2, v3), (3, v4), (4, v5), (5, v6), (6, v7), (7, v8), (8, v9), (9, v10), (10, v11), (11, v12), (12, v13), (13, v14), (14, v15), (15, v16), (16, v17), (17, v18), (18, v19), (19, v20));

/// Field path supporting up to 5 levels of nesting
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct FieldPath(String);

impl std::fmt::Display for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FieldPath {
    /// Creates a new field path from a string-like value
    ///
    /// # Arguments
    /// * `path` - Field path as a string
    ///
    /// # Errors
    /// Returns ValidationError if path exceeds 256 characters
    ///
    /// # Examples
    /// ```
    /// # use lambda_appsync::{AppsyncError, subscription_filters::FieldPath};
    /// # fn example() -> Result<(), AppsyncError> {
    /// let path = FieldPath::new("user.profile.name")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: impl Into<String>) -> Result<Self, AppsyncError> {
        let path = path.into();
        if path.len() > 256 {
            return Err(AppsyncError::new(
                "ValidationError",
                "Field path exceeds 256 characters",
            ));
        }
        // Could add more validation here
        Ok(Self(path))
    }

    /// Creates a new field path from a string-like value without validation
    ///
    /// # Safety
    /// This function skips validation of the field path. The caller must ensure:
    /// - Path length does not exceed 256 characters
    /// - Path contains valid field references only
    /// - Path nesting depth does not exceed 5 levels
    ///
    /// # Examples
    /// ```
    /// # use lambda_appsync::subscription_filters::FieldPath;
    /// let path = unsafe { FieldPath::new_unchecked("user.name") };
    /// ```
    pub unsafe fn new_unchecked(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    // IFSB operators
    /// Creates an equality filter
    pub fn eq<IFSB: IFSBValueMarker>(self, ifsb: IFSB) -> FieldFilter {
        FieldFilter::new(self, ifsb.to_value(), FilterOp::Eq)
    }

    /// Creates an equality filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    /// - A boolean
    pub unsafe fn eq_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(self, serde_json::to_value(value).unwrap(), FilterOp::Eq)
    }

    /// Creates a not equal filter
    pub fn ne<IFSB: IFSBValueMarker>(self, ifsb: IFSB) -> FieldFilter {
        FieldFilter::new(self, ifsb.to_value(), FilterOp::Ne)
    }

    /// Creates a not equal filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    /// - A boolean
    pub unsafe fn ne_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(self, serde_json::to_value(value).unwrap(), FilterOp::Ne)
    }

    // IFS operators
    /// Creates a less than or equal filter
    pub fn le<IFS: IFSValueMarker>(self, ifs: IFS) -> FieldFilter {
        FieldFilter::new(self, ifs.to_value(), FilterOp::Le)
    }

    /// Creates a less than or equal filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn le_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(self, serde_json::to_value(value).unwrap(), FilterOp::Le)
    }

    /// Creates a less than filter
    pub fn lt<IFS: IFSValueMarker>(self, ifs: IFS) -> FieldFilter {
        FieldFilter::new(self, ifs.to_value(), FilterOp::Lt)
    }

    /// Creates a less than filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn lt_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(self, serde_json::to_value(value).unwrap(), FilterOp::Lt)
    }

    /// Creates a greater than or equal filter
    pub fn ge<IFS: IFSValueMarker>(self, ifs: IFS) -> FieldFilter {
        FieldFilter::new(self, ifs.to_value(), FilterOp::Ge)
    }

    /// Creates a greater than or equal filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn ge_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(self, serde_json::to_value(value).unwrap(), FilterOp::Ge)
    }

    /// Creates a greater than filter
    pub fn gt<IFS: IFSValueMarker>(self, ifs: IFS) -> FieldFilter {
        FieldFilter::new(self, ifs.to_value(), FilterOp::Gt)
    }

    /// Creates a greater than filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn gt_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(self, serde_json::to_value(value).unwrap(), FilterOp::Gt)
    }

    /// Creates a contains filter for strings or arrays
    pub fn contains<IFS: IFSValueMarker>(self, ifs: IFS) -> FieldFilter {
        FieldFilter::new(self, ifs.to_value(), FilterOp::Contains)
    }

    /// Creates a contains filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn contains_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(
            self,
            serde_json::to_value(value).unwrap(),
            FilterOp::Contains,
        )
    }

    /// Creates a not contains filter for strings or arrays
    pub fn not_contains<IFS: IFSValueMarker>(self, ifs: IFS) -> FieldFilter {
        FieldFilter::new(self, ifs.to_value(), FilterOp::NotContains)
    }

    /// Creates a not contains filter from any serializable value
    ///
    /// # Safety
    /// The caller must ensure the value serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn not_contains_unchecked<T: Serialize>(self, value: T) -> FieldFilter {
        FieldFilter::new(
            self,
            serde_json::to_value(value).unwrap(),
            FilterOp::NotContains,
        )
    }

    // String only
    /// Creates a begins with filter for string fields
    pub fn begins_with(self, value: impl Into<String>) -> FieldFilter {
        FieldFilter::new(
            self,
            serde_json::Value::String(value.into()),
            FilterOp::BeginsWith,
        )
    }

    // Array operators
    /// Creates an IN filter accepting up to 5 values
    ///
    /// # Examples
    /// ```
    /// # use lambda_appsync::{AppsyncError, subscription_filters::FieldPath};
    /// # fn example() -> Result<(), AppsyncError> {
    /// let path = FieldPath::new("user.id")?;
    /// let filter = path.in_values(["id1", "id2", "id3"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn in_values<IFS: IFSValueMarker>(self, values: impl Into<InVec<IFS>>) -> FieldFilter {
        let in_vec = values.into();
        FieldFilter::new(self, in_vec.to_value(), FilterOp::In)
    }

    /// Creates an IN filter from any array of up to 5 serializable values
    ///
    /// # Safety
    /// The caller must ensure each value in the array serializes to either:
    /// - A number
    /// - A string
    pub unsafe fn in_values_unchecked<T: Serialize>(
        self,
        values: impl Into<InVec<T>>,
    ) -> FieldFilter {
        let in_vec = values.into();
        FieldFilter::new(self, serde_json::to_value(in_vec).unwrap(), FilterOp::In)
    }

    /// Creates a NOT IN filter accepting up to 5 values
    ///
    /// # Examples
    /// ```
    /// # use lambda_appsync::{AppsyncError, subscription_filters::FieldPath};
    /// # fn example() -> Result<(), AppsyncError> {
    /// let path = FieldPath::new("user.role")?;
    /// let filter = path.not_in(["admin", "moderator"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn not_in<IFS: IFSValueMarker>(self, values: impl Into<InVec<IFS>>) -> FieldFilter {
        let in_vec = values.into();
        FieldFilter::new(self, in_vec.to_value(), FilterOp::NotIn)
    }

    /// Creates a NOT IN filter from any array of up to 5 serializable values
    ///
    /// # Safety
    /// The caller must ensure values in the array serializes to either:
    /// - Numbers
    /// - Strings
    pub unsafe fn not_in_unchecked<T: Serialize>(self, values: impl Into<InVec<T>>) -> FieldFilter {
        let in_vec = values.into();
        FieldFilter::new(self, serde_json::to_value(in_vec).unwrap(), FilterOp::NotIn)
    }

    /// Creates a BETWEEN filter that matches values in a range
    pub fn between<IFS: IFSValueMarker>(self, start: IFS, end: IFS) -> FieldFilter {
        FieldFilter::new(
            self,
            FixedVec([Some(start), Some(end)]).to_value(),
            FilterOp::Between,
        )
    }

    /// Creates a BETWEEN filter from any two serializable values
    ///
    /// # Safety
    /// The caller must ensure both values serialize to either:
    /// - Numbers
    /// - Strings
    pub unsafe fn between_unchecked<T: Serialize>(self, start: T, end: T) -> FieldFilter {
        FieldFilter::new(
            self,
            serde_json::to_value(FixedVec([Some(start), Some(end)])).unwrap(),
            FilterOp::Between,
        )
    }

    /// Creates a contains any filter accepting up to 20 values
    ///
    /// # Examples
    /// ```
    /// # use lambda_appsync::{AppsyncError, subscription_filters::FieldPath};
    /// # fn example() -> Result<(), AppsyncError> {
    /// let path = FieldPath::new("user.permissions")?;
    /// let filter = path.contains_any(["read", "write", "delete"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_any<IFS: IFSValueMarker>(
        self,
        values: impl Into<ContainsAnyVec<IFS>>,
    ) -> FieldFilter {
        let contains_vec = values.into();
        FieldFilter::new(self, contains_vec.to_value(), FilterOp::ContainsAny)
    }

    /// Creates a contains any filter from any array of up to 20 serializable values
    ///
    /// # Safety
    /// The caller must ensure values in the array serializes to either:
    /// - Numbers
    /// - Strings
    pub unsafe fn contains_any_unchecked<T: Serialize>(
        self,
        values: impl Into<ContainsAnyVec<T>>,
    ) -> FieldFilter {
        let contains_vec = values.into();
        FieldFilter::new(
            self,
            serde_json::to_value(contains_vec).unwrap(),
            FilterOp::ContainsAny,
        )
    }
}

/// AppSync subscription filter comparison operator, serialized in camelCase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
enum FilterOp {
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
    Contains,
    NotContains,
    BeginsWith,
    In,
    NotIn,
    Between,
    ContainsAny,
}

/// A single field filter that combines a field path with an operator and value
/// in the AppSync subscription filter format.
///
/// Should be created from a [FieldPath] using the operator methods:
///
/// # Example
/// ```
/// # use lambda_appsync::{AppsyncError, subscription_filters::{FieldPath, FieldFilter}};
/// # fn example() -> Result<FieldFilter, AppsyncError> {
/// let path = FieldPath::new("user.name")?;
/// let filter = path.eq("example");
/// # Ok(filter)
/// # }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct FieldFilter {
    #[serde(rename = "fieldName")]
    path: FieldPath,
    operator: FilterOp,
    value: serde_json::Value,
}
impl FieldFilter {
    /// Constructs a [`FieldFilter`] from a path, pre-serialized value, and operator.
    fn new(path: FieldPath, value: serde_json::Value, operator: FilterOp) -> Self {
        Self {
            path,
            value,
            operator,
        }
    }
}
/// A single filter limited to 5 field filters
///
/// Can be created from an arrays of up to 5 [FieldFilter] elements.
///
/// # Example
/// ```
/// # use lambda_appsync::{subscription_filters::{FieldPath, Filter}, AppsyncError};
/// # fn example() -> Result<Filter, AppsyncError> {
/// let filter = Filter::from([
///     FieldPath::new("user.name")?.eq("test"),
///     FieldPath::new("user.age")?.gt(21),
///     FieldPath::new("user.role")?.in_values(["admin", "moderator"])
/// ]);
/// # Ok(filter)
/// # }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Filter {
    filters: FixedVec<FieldFilter, 5>,
}

impl<T> From<T> for Filter
where
    T: Into<FixedVec<FieldFilter, 5>>,
{
    fn from(filters: T) -> Self {
        Self {
            filters: filters.into(),
        }
    }
}

impl From<FieldFilter> for Filter {
    fn from(value: FieldFilter) -> Self {
        Filter::from([value])
    }
}

/// A filter group limited to 10 filters combined with OR logic
///
/// Can be created from an arrays of up to 10 [Filter] elements.
///
/// # Example
/// ```
/// # use lambda_appsync::{subscription_filters::{FieldPath, Filter, FilterGroup}, AppsyncError};
/// # fn example() -> Result<FilterGroup, AppsyncError> {
/// let group = FilterGroup::from([
///     // First filter - user is admin AND age over 21
///     Filter::from([
///         FieldPath::new("user.role")?.eq("admin"),
///         FieldPath::new("user.age")?.gt(21)
///     ]),
///     // Second filter - user is moderator AND has required permissions
///     Filter::from([
///         FieldPath::new("user.role")?.eq("moderator"),
///         FieldPath::new("user.permissions")?.contains_any(["moderate", "review"])
///     ])
/// ]);
/// # Ok(group)
/// # }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct FilterGroup {
    #[serde(rename = "filterGroup")]
    filters: FixedVec<Filter, 10>,
}

impl<T> From<T> for FilterGroup
where
    T: Into<FixedVec<Filter, 10>>,
{
    fn from(filters: T) -> Self {
        Self {
            filters: filters.into(),
        }
    }
}

impl From<FieldFilter> for FilterGroup {
    fn from(value: FieldFilter) -> Self {
        FilterGroup::from(Filter::from([value]))
    }
}
impl From<Filter> for FilterGroup {
    fn from(value: Filter) -> Self {
        FilterGroup::from([value])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn filter_value(f: &FieldFilter) -> serde_json::Value {
        serde_json::to_value(f).unwrap()
    }

    #[test]
    fn test_create_paths() {
        let path = FieldPath::new("user.name").unwrap();
        assert_eq!(path.to_string(), "user.name");

        let path = FieldPath::new("nested.one.two.three.four.five");
        assert!(path.is_ok());

        let long_path = "a".repeat(257);
        assert!(FieldPath::new(long_path).is_err());
    }

    #[test]
    fn test_eq_operator() {
        // Test string equality
        let filter = FieldPath::new("service").unwrap().eq("AWS AppSync");
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "service",
                "operator": "eq",
                "value": "AWS AppSync"
            })
        );

        // Test numeric equality
        let filter = FieldPath::new("severity").unwrap().eq(5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "severity",
                "operator": "eq",
                "value": 5
            })
        );

        // Test boolean equality
        let filter = FieldPath::new("enabled").unwrap().eq(true);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "enabled",
                "operator": "eq",
                "value": true
            })
        );
    }

    #[test]
    fn test_ne_operator() {
        // Test string inequality
        let filter = FieldPath::new("service").unwrap().ne("AWS AppSync");
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "service",
                "operator": "ne",
                "value": "AWS AppSync"
            })
        );

        // Test numeric inequality
        let filter = FieldPath::new("severity").unwrap().ne(5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "severity",
                "operator": "ne",
                "value": 5
            })
        );

        // Test boolean inequality
        let filter = FieldPath::new("enabled").unwrap().ne(true);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "enabled",
                "operator": "ne",
                "value": true
            })
        );
    }

    #[test]
    fn test_comparison_operators() {
        let path = FieldPath::new("size").unwrap();

        // Test le
        let filter = path.clone().le(5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "size",
                "operator": "le",
                "value": 5
            })
        );

        // Test lt
        let filter = path.clone().lt(5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "size",
                "operator": "lt",
                "value": 5
            })
        );

        // Test ge
        let filter = path.clone().ge(5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "size",
                "operator": "ge",
                "value": 5
            })
        );

        // Test gt
        let filter = path.gt(5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "size",
                "operator": "gt",
                "value": 5
            })
        );
    }

    #[test]
    fn test_contains_operators() {
        let path = FieldPath::new("seats").unwrap();

        // Test contains with number
        let filter = path.clone().contains(10);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "seats",
                "operator": "contains",
                "value": 10
            })
        );

        // Test contains with string
        let event_path = FieldPath::new("event").unwrap();
        let filter = event_path.clone().contains("launch");
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "event",
                "operator": "contains",
                "value": "launch"
            })
        );

        // Test notContains
        let filter = path.not_contains(10);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "seats",
                "operator": "notContains",
                "value": 10
            })
        );
    }

    #[test]
    fn test_begins_with() {
        let filter = FieldPath::new("service").unwrap().begins_with("AWS");
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "service",
                "operator": "beginsWith",
                "value": "AWS"
            })
        );
    }

    #[test]
    fn test_in_operators() {
        let path = FieldPath::new("severity").unwrap();

        // Test in with up to 5 values
        let filter = path.clone().in_values([1, 2, 3]);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "severity",
                "operator": "in",
                "value": [1, 2, 3]
            })
        );

        // Test notIn with up to 5 values
        let filter = path.not_in([1, 2, 3]);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "severity",
                "operator": "notIn",
                "value": [1, 2, 3]
            })
        );
    }

    #[test]
    fn test_between() {
        let filter = FieldPath::new("severity").unwrap().between(1, 5);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "severity",
                "operator": "between",
                "value": [1, 5]
            })
        );
    }

    #[test]
    fn test_contains_any() {
        let filter = FieldPath::new("seats").unwrap().contains_any([10, 15, 20]);
        assert_eq!(
            filter_value(&filter),
            json!({
                "fieldName": "seats",
                "operator": "containsAny",
                "value": [10, 15, 20]
            })
        );
    }

    #[test]
    fn test_filter_group() {
        let group = FilterGroup::from([Filter::from([
            FieldPath::new("userId").unwrap().eq(1),
            FieldPath::new("group")
                .unwrap()
                .in_values(["Admin", "Developer"]),
        ])]);

        assert_eq!(
            serde_json::to_value(group).unwrap(),
            json!({
                "filterGroup": [
                    {
                        "filters": [
                            {
                                "fieldName": "userId",
                                "operator": "eq",
                                "value": 1
                            },
                            {
                                "fieldName": "group",
                                "operator": "in",
                                "value": ["Admin", "Developer"]
                            }
                        ]
                    }
                ]
            })
        );
    }

    #[test]
    fn test_complex_filter() {
        let group = FilterGroup::from([
            Filter::from([
                FieldPath::new("severity").unwrap().le(3),
                FieldPath::new("type").unwrap().eq("error"),
            ]),
            Filter::from([
                FieldPath::new("service").unwrap().begins_with("AWS"),
                FieldPath::new("region")
                    .unwrap()
                    .in_values(["us-east-1", "eu-west-1"]),
            ]),
        ]);

        assert_eq!(
            serde_json::to_value(group).unwrap(),
            json!({
                "filterGroup": [
                    {
                        "filters": [
                            {
                                "fieldName": "severity",
                                "operator": "le",
                                "value": 3
                            },
                            {
                                "fieldName": "type",
                                "operator": "eq",
                                "value": "error"
                            }
                        ]
                    },
                    {
                        "filters": [
                            {
                                "fieldName": "service",
                                "operator": "beginsWith",
                                "value": "AWS"
                            },
                            {
                                "fieldName": "region",
                                "operator": "in",
                                "value": ["us-east-1", "eu-west-1"]
                            }
                        ]
                    }
                ]
            })
        );
    }
}
