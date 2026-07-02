use crate::{Error, Result};

/// Resource limits applied while parsing untrusted bencode.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseLimits {
    max_depth: usize,
    max_items: usize,
    max_byte_string_length: usize,
    max_integer_digits: usize,
    max_total_input: usize,
    max_owned_allocation: usize,
}

/// Options applied while loading and owning metainfo.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ParseOptions {
    limits: ParseLimits,
}

impl ParseOptions {
    /// Creates load options from a complete resource-limit configuration.
    #[must_use]
    pub const fn new(limits: ParseLimits) -> Self {
        Self { limits }
    }

    /// Returns the configured parsing and ownership limits.
    #[must_use]
    pub const fn limits(self) -> ParseLimits {
        self.limits
    }
}

#[derive(Debug)]
pub(crate) struct AllocationBudget {
    limits: ParseLimits,
    used: usize,
}

impl AllocationBudget {
    pub(crate) const fn new(limits: ParseLimits) -> Self {
        Self { limits, used: 0 }
    }

    pub(crate) fn charge(&mut self, amount: usize) -> Result<()> {
        self.used = self.limits.checked_owned_allocation(self.used, amount)?;
        Ok(())
    }
}

impl ParseLimits {
    /// Creates a complete parser limit configuration.
    #[must_use]
    pub const fn new(
        max_depth: usize,
        max_items: usize,
        max_byte_string_length: usize,
        max_total_input: usize,
        max_owned_allocation: usize,
    ) -> Self {
        Self {
            max_depth,
            max_items,
            max_byte_string_length,
            max_integer_digits: 4_096,
            max_total_input,
            max_owned_allocation,
        }
    }

    /// Returns the maximum container nesting depth.
    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// Returns the maximum number of parsed values.
    #[must_use]
    pub const fn max_items(self) -> usize {
        self.max_items
    }

    /// Returns the maximum byte-string length.
    #[must_use]
    pub const fn max_byte_string_length(self) -> usize {
        self.max_byte_string_length
    }

    /// Returns the maximum number of integer digits, excluding a minus sign.
    #[must_use]
    pub const fn max_integer_digits(self) -> usize {
        self.max_integer_digits
    }

    /// Replaces the maximum integer digit count.
    #[must_use]
    pub const fn with_max_integer_digits(mut self, maximum: usize) -> Self {
        self.max_integer_digits = maximum;
        self
    }

    /// Returns the maximum total input length.
    #[must_use]
    pub const fn max_total_input(self) -> usize {
        self.max_total_input
    }

    /// Returns the maximum budget for allocations made by owned conversion.
    #[must_use]
    pub const fn max_owned_allocation(self) -> usize {
        self.max_owned_allocation
    }

    /// Checks a nesting depth against the configured maximum.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when `actual` exceeds the maximum depth.
    pub const fn check_depth(self, actual: usize) -> Result<()> {
        check_limit("depth", actual, self.max_depth)
    }

    /// Checks an item count against the configured maximum.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when `actual` exceeds the maximum count.
    pub const fn check_items(self, actual: usize) -> Result<()> {
        check_limit("item count", actual, self.max_items)
    }

    /// Checks a byte-string length against the configured maximum.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when `actual` exceeds the maximum length.
    pub const fn check_byte_string_length(self, actual: usize) -> Result<()> {
        check_limit("byte-string length", actual, self.max_byte_string_length)
    }

    /// Checks an integer digit count against the configured maximum.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when `actual` exceeds the maximum count.
    pub const fn check_integer_digits(self, actual: usize) -> Result<()> {
        check_limit("integer digits", actual, self.max_integer_digits)
    }

    /// Checks total input length against the configured maximum.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when `actual` exceeds the input maximum.
    pub const fn check_total_input(self, actual: usize) -> Result<()> {
        check_limit("total input", actual, self.max_total_input)
    }

    /// Adds an allocation to an accumulated total without overflow.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when addition overflows or the resulting
    /// total exceeds the owned-allocation budget.
    pub const fn checked_owned_allocation(self, current: usize, added: usize) -> Result<usize> {
        let Some(total) = current.checked_add(added) else {
            return Err(Error::resource_limit(
                "owned allocation",
                usize::MAX,
                self.max_owned_allocation,
            ));
        };
        if total > self.max_owned_allocation {
            Err(Error::resource_limit(
                "owned allocation",
                total,
                self.max_owned_allocation,
            ))
        } else {
            Ok(total)
        }
    }
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self::new(
            128,
            1_000_000,
            128 * 1024 * 1024,
            256 * 1024 * 1024,
            256 * 1024 * 1024,
        )
    }
}

const fn check_limit(limit: &'static str, actual: usize, maximum: usize) -> Result<()> {
    if actual > maximum {
        Err(Error::resource_limit(limit, actual, maximum))
    } else {
        Ok(())
    }
}
